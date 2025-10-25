use lotus_extra::vehicle::CockpitSide;
use pandemist_vehicle_elements::{
    api::light::Light,
    components::gt6n::{foldingramp::Foldingramp, lift::Hublift},
    elements::tech::{
        buttons::PushButton,
        switches::{StepSwitch, SwitchEventAction},
    },
    management::{communicator::Com, enums::general_enums::CabActivState},
    messages::diagnostic_messages::{DiagnosticFaultKind, DiagnosticMessageSender},
};

use crate::general::{
    local_values::{WslCabState, WslLighttest, WslLowVoltageNorm},
    setup::{const_veh_variant, FahrzeugVariante},
};

enum UsedBoardinghelper {
    WheelchairLift(Box<WheelchairLiftGt6n>),
    WheelchairRamp(Box<WheelchairRampGt6n>),
}

pub struct WheelchairHelper {
    helper: UsedBoardinghelper,
}

impl WheelchairHelper {
    pub fn new() -> Self {
        let helper = if const_veh_variant() != FahrzeugVariante::Gt6u {
            UsedBoardinghelper::WheelchairLift(Box::default())
        } else {
            UsedBoardinghelper::WheelchairRamp(Box::default())
        };
        Self { helper }
    }

    pub fn tick(&mut self, allowed: bool, com: &mut Com) {
        match &mut self.helper {
            UsedBoardinghelper::WheelchairLift(lift) => {
                lift.tick(allowed, com);
            }
            UsedBoardinghelper::WheelchairRamp(ramp) => {
                ramp.tick(allowed);
            }
        }
    }

    pub fn in_use(&mut self) -> bool {
        match &self.helper {
            UsedBoardinghelper::WheelchairLift(lift) => lift.in_use(),
            UsedBoardinghelper::WheelchairRamp(ramp) => ramp.in_use(),
        }
    }

    pub fn blink_relais(&mut self) -> bool {
        match &self.helper {
            UsedBoardinghelper::WheelchairLift(lift) => lift.lift.warnrelais.is_on,
            UsedBoardinghelper::WheelchairRamp(ramp) => ramp.rampe.warnrelais.is_on,
        }
    }

    pub fn lift_requested(&mut self) -> bool {
        match &self.helper {
            UsedBoardinghelper::WheelchairLift(lift) => lift.lift_requested,
            UsedBoardinghelper::WheelchairRamp(_) => false,
        }
    }
}

impl Default for WheelchairHelper {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WheelchairRampGt6n {
    rampe: Foldingramp,
}

impl WheelchairRampGt6n {
    fn new() -> Self {
        Self {
            rampe: Foldingramp::new("", CockpitSide::A),
        }
    }

    fn tick(&mut self, allowed: bool) {
        self.rampe.tick(allowed);
    }
    fn in_use(&self) -> bool {
        self.rampe.in_use
    }
}

impl Default for WheelchairRampGt6n {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WheelchairLiftGt6n {
    mms_fault_sender: DiagnosticMessageSender,
    lift: Hublift,

    sw_lift_up_down: StepSwitch,
    sw_lift_level_target: StepSwitch,
    btn_lift_emergency_off: PushButton,

    pub lift_requested: bool,

    lm_emergency_off: Light,
}

impl WheelchairLiftGt6n {
    fn new() -> Self {
        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),
            lift: Hublift::new(CockpitSide::A),
            sw_lift_up_down: StepSwitch::builder(
                "AV_A_Sw_Hublift_HebenSenken",
                Some(CockpitSide::A),
            )
            .event("Hublift_Heben", SwitchEventAction::Plus)
            .event("Hublift_Senken", SwitchEventAction::Minus)
            .event("DisplayMoveSelUp", SwitchEventAction::Plus)
            .event("DisplayMoveSelDn", SwitchEventAction::Minus)
            .snd_default_plus("Snd_A_Switch")
            .snd_default_minus("Snd_A_Switch")
            .min(-1)
            .max(1)
            .min_spring()
            .max_spring()
            .build(),
            sw_lift_level_target: StepSwitch::builder(
                "AV_A_Sw_Hublift_Hoehenvorgabe",
                Some(CockpitSide::A),
            )
            .event("Hublift_Bahnsteig", SwitchEventAction::Plus)
            .event("Hublift_Strasse", SwitchEventAction::Minus)
            .snd_default_plus("Snd_A_Switch")
            .snd_default_minus("Snd_A_Switch")
            .min(-1)
            .max(1)
            .build(),

            btn_lift_emergency_off: PushButton::builder(
                "AV_A_Btn_Hublift_Notablage",
                "Hublift_Notablegen",
                Some(CockpitSide::A),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),

            lm_emergency_off: Light::new(Some("LM_A_Notablegen")),

            lift_requested: false,
        }
    }

    fn tick(&mut self, allowed: bool, com: &mut Com) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let cab_a_runmode =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Star;

        // Read fuses
        let fuse_power = com.fuse.is_on("HubliftLeistungsteil");
        let fuse_control = com.fuse.is_on("HubliftSteuersignal");
        let sicherung_kompressor = com.fuse.is_on("HubliftKompressor");

        // Input from key events
        self.sw_lift_up_down.tick();
        self.sw_lift_level_target.tick();
        self.btn_lift_emergency_off.tick();

        // Input - Signale

        // Main logic
        let lift_target = match self.sw_lift_level_target.value(cab_a_runmode && allowed) {
            1 => 2,
            -1 => 3,
            _ => 0,
        };

        self.lift_requested = self.sw_lift_level_target.value(cab_a_runmode) != 0;

        self.lift.fuse_control = fuse_control;
        self.lift.fuse_power = fuse_power;
        self.lift.fuse_control_signal = true;

        self.lift.tick(
            self.sw_lift_up_down.value(cab_a_runmode),
            lift_target,
            allowed,
            self.btn_lift_emergency_off.value(cab_a_runmode),
            voltage,
        );

        // Assign output
        self.lm_emergency_off
            .set_brightness(voltage * (light_test as u8 as f32));

        //===============================================================
        // MMS communication
        //===============================================================

        self.mms_fault_sender.send(
            DiagnosticFaultKind::HubliftDefektA,
            !(fuse_power && fuse_control && sicherung_kompressor),
            Some(CockpitSide::A),
        );
    }

    fn in_use(&self) -> bool {
        self.lift.in_use
    }
}

impl Default for WheelchairLiftGt6n {
    fn default() -> Self {
        Self::new()
    }
}
