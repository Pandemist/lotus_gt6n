use lotus_extra::{messages::pis::RoutingDirection, vehicle::CockpitSide};
use lotus_script::{
    prelude::{send_message, Message, MessageTarget},
    time::delta,
};
use pandemist_vehicle_elements::{
    api::{
        light::Light,
        variable::{get_var, set_var},
    },
    components::general::switch_control_unit::{SwitchSender, SwtichControlUnit},
    elements::tech::{
        buttons::PushButton,
        handpin::HandPin,
        key_switch::{KeyDepot, KeySwitch},
        seals::SealedSwitch,
        switches::{StepSwitch, Switch, SwitchEventAction},
    },
    management::{
        communicator::Com,
        enums::{general_enums::CabActivState, state_enums::ChangedState},
        structs::general_structs::TrainActivState,
    },
    messages::diagnostic_messages::{DiagnosticFaultKind, DiagnosticMessageSender},
};

use crate::general::{
    local_values::{
        WslBatteryMainSwitch, WslCabIndicatorBrightness, WslLighttest, WslLowVoltageNorm,
        WslTrainState, WslWorkshopKey,
    },
    setup::past_date_750v,
};

pub struct SimpleComponents {
    mms_fault_sender: DiagnosticMessageSender,

    a_sw_wheel_lubrication: Switch,
    a_sw_ohne_name: Switch,
    a_sw_bypass_switch_kwr_plombe: SealedSwitch,

    a_key_workshop: KeySwitch,
    a_btn_notstart: PushButton,
    a_sw_wasch_run: Switch,

    a_sw_loudspeaker: StepSwitch,

    a_sw_switching_control: HandPin,
    switch_direction: RoutingDirection,
    switch_timer: f32,

    a_lm_emergency_start: Light,

    switch_control_unit: SwtichControlUnit,
}

impl SimpleComponents {
    pub fn new(working_key: KeyDepot) -> Self {
        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),

            a_sw_wheel_lubrication: Switch::builder(
                "AV_A_Sw_Spurkranzschmierung",
                Some(CockpitSide::A),
            )
            .snd_toggle("Snd_A_Switch")
            .event_toggle("Spurkranzschmierung")
            .build(),

            a_sw_ohne_name: Switch::builder("AV_A_Sw_ohneName", Some(CockpitSide::A))
                .snd_toggle("Snd_A_Switch")
                .event_toggle("NoNameSwitch")
                .build(),

            a_sw_bypass_switch_kwr_plombe: SealedSwitch::new(
                Some(CockpitSide::A),
                "vis_A_Plombe_Hilfsschalter_KWR",
                "Plombe_Hilfsschalter_KWR",
                Switch::builder("AV_A_sw_bypass_switch_KWR", Some(CockpitSide::A))
                    .snd_toggle("Snd_A_Switch")
                    .event_toggle("Hilfsschalter_KWR")
                    .build(),
            ),

            a_key_workshop: KeySwitch::builder(
                working_key,
                "AV_A_Key_Werkstatt",
                "vis_A_Key_Werkstatt",
                Some(CockpitSide::A),
            )
            .event_toggle("Key_Werkstatt_Toggle")
            .event_turn("Key_Werkstatt_Turn")
            .snd_insert("Snd_A_Key_Insert")
            .snd_takeout("Snd_A_Key_Takeout")
            .snd_default("Snd_A_Key_Turn")
            .pullout_min()
            .pullout_max()
            .build(),

            a_btn_notstart: PushButton::builder(
                "AV_A_Btn_Notstart",
                "Notstart",
                Some(CockpitSide::A),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),
            a_sw_wasch_run: Switch::builder("AV_A_Sw_Waschfahrt", Some(CockpitSide::A))
                .snd_toggle("Snd_A_Switch")
                .event_toggle("Waschfahrt_Toggle")
                .build(),

            a_sw_loudspeaker: StepSwitch::builder("AV_A_Sw_Lautsprecher", Some(CockpitSide::A))
                .event("Lautsprecher_Plus", SwitchEventAction::Plus)
                .event("Lautsprecher_Minus", SwitchEventAction::Minus)
                .snd_default_plus("Snd_A_Switch")
                .snd_default_minus("Snd_A_Switch")
                .min(-1)
                .max(1)
                .min_spring()
                .max_spring()
                .build(),

            a_sw_switching_control: HandPin::builder(
                "AV_A_Sw_Weichenpin_X",
                "AV_A_Sw_Weichenpin_Y",
                Some(CockpitSide::A),
            )
            .mouse_factor(0.15)
            .event_grab("Weichenschalter")
            .event_override_n("SwitchStraight")
            .event_override_s("Trigger_Request")
            .event_override_e("SwitchRight")
            .event_override_w("SwitchLeft")
            .build(),

            switch_direction: RoutingDirection::Off,
            switch_timer: -1.0,

            a_lm_emergency_start: Light::new(Some("LM_A_Notstart")),

            switch_control_unit: SwtichControlUnit::new(
                vec![SwitchSender::Vehicle, SwitchSender::Modul(0)],
                0,
            ),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        self.fuse_to_mms(com);

        // Read local signals
        let battery_switch =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let cab_a_activ = com
            .lv
            .get_or(WslTrainState, TrainActivState::default())
            .cab_a
            > CabActivState::Off;
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let cab_indicator_light_level = com.lv.get_or(WslCabIndicatorBrightness(0), 1.0);

        // Read fuses

        // Input from key events
        self.a_sw_wheel_lubrication.tick();
        self.a_sw_ohne_name.tick();
        self.a_sw_bypass_switch_kwr_plombe.tick();

        self.a_key_workshop.tick();

        com.lv.set(
            WslWorkshopKey(0),
            self.a_key_workshop.value(battery_switch) > 0,
        );

        self.a_btn_notstart.tick();
        self.a_sw_wasch_run.tick();

        self.a_sw_loudspeaker.tick();

        self.a_sw_switching_control.tick();

        // Handle switchrequests
        if self.a_sw_switching_control.direction.up
            && self.switch_direction != RoutingDirection::Straight
        {
            self.switch_timer = 10.0;
            self.switch_direction = RoutingDirection::Straight;
        }

        if self.a_sw_switching_control.direction.right
            && self.switch_direction != RoutingDirection::Right
        {
            self.switch_timer = 10.0;
            self.switch_direction = RoutingDirection::Right;
        }

        if self.a_sw_switching_control.direction.left
            && self.switch_direction != RoutingDirection::Left
        {
            self.switch_timer = 10.0;
            self.switch_direction = RoutingDirection::Left;
        }

        if self.switch_timer > 0.0 {
            self.switch_timer -= delta();
        }

        if self.switch_timer < 0.0 && self.switch_direction != RoutingDirection::Off {
            self.switch_direction = RoutingDirection::Off;

            send_message(
                &(self.switch_direction),
                [MessageTarget::Broadcast {
                    across_couplings: false,
                    include_self: true,
                }],
            );
        }

        // switiching control
        self.switch_control_unit
            .tick(cab_a_activ, cab_a_activ, cab_a_activ);

        // Z-Position
        let z_stellung = get_var::<f32>("ZStellung_C")
            + 0.1
                * (get_var::<f32>("ZStellung_A") + get_var::<f32>("ZStellung_B")
                    - get_var::<f32>("ZStellung_C"));

        set_var("ZStellung_C", z_stellung);

        // Assign output
        self.a_lm_emergency_start
            .set_brightness(voltage * cab_indicator_light_level * (light_test as u8 as f32));

        //===============================================================
        // MMS communication
        //===============================================================

        self.mms_fault_sender.send(
            DiagnosticFaultKind::KwrUeberbrueckt,
            self.a_sw_bypass_switch_kwr_plombe.switch.value(cab_a_activ),
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Waschfahrt,
            self.a_sw_wasch_run.value(cab_a_activ),
            None,
        );
    }

    fn fuse_to_mms(&mut self, com: &mut Com) {
        // Sicherungskreis 4
        self.mms_fault_sender.send(
            DiagnosticFaultKind::Sicherungskreis4aA,
            !(com.fuse.is_on("IKEELAversorgung")
                && com.fuse.is_on("IKEaufruesten")
                && com.fuse.is_on("KWRsifaEingangssingal")
                && past_date_750v()),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Sicherungskreis4bA,
            !(com.fuse.is_on("IKEELAversorgung")
                && com.fuse.is_on("IKEaufruesten")
                && com.fuse.is_on("KWRsifaEingangssingal"))
                && past_date_750v(),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Sicherungskreis4cA,
            !(com.fuse.is_on("FunkIMUVersorgung")),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Sicherungskreis4dA,
            !(com.fuse.is_on("Notruf")),
            Some(CockpitSide::A),
        );

        // Sicherungskreis 7
        self.mms_fault_sender.send(
            DiagnosticFaultKind::Sicherungskreis7aA,
            !(com.fuse.is_on("Frontscheibenheizung") && com.fuse.is_on("Spurkranzschmierung")),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Sicherungskreis7bA,
            !(com.fuse.is_on("SeitenscheibenheizungSteuerung")),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Sicherungskreis7cA,
            !(com.fuse.is_on("Sandrohrheizung")),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Sicherungskreis8aA,
            !(com.fuse.is_on("VersorgungTuer1") && com.fuse.is_on("SteuerungTuer1")),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Sicherungskreis8bA,
            !(com.fuse.is_on("ZentraleTuersteuerung")),
            Some(CockpitSide::A),
        );
    }

    pub fn on_message(&mut self, msg: Message) {
        self.switch_control_unit.on_message(msg.clone());
    }
}
