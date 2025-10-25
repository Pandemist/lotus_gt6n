use lotus_extra::vehicle::CockpitSide;
use lotus_script::time::delta;
use pandemist_vehicle_elements::{
    api::sound::Sound,
    elements::tech::{
        buttons::PushButton,
        switches::{StepSwitch, Switch, SwitchEventAction},
    },
    management::{communicator::Com, enums::state_enums::ChangedState},
    messages::diagnostic_messages::{DiagnosticFaultKind, DiagnosticMessageSender},
};

use crate::general::local_values::WslBatteryMainSwitch;

pub struct HeatingVentilation {
    mms_fault_sender: DiagnosticMessageSender,

    a_sw_heater_passenger: Switch,
    a_sw_heater_driver: Switch,
    a_sw_parking_heater: StepSwitch,
    a_sw_driver_seat_heater: Switch,
    a_sw_mirror_heater: Switch,
    a_btn_window_heater: PushButton,

    a_window_heater_timer: f32,
    a_window_heater_relay: bool,
    a_window_heater_relay_last: bool,

    a_snd_window_heater_relay_on: Sound,
    a_snd_window_heater_relay_off: Sound,
}

impl HeatingVentilation {
    pub fn new() -> Self {
        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),

            a_sw_heater_passenger: Switch::builder("AV_A_Sw_Heizung_FGR", Some(CockpitSide::A))
                .event_toggle("Heizung_FGR")
                .snd_toggle("Snd_A_Switch")
                .build(),

            a_sw_heater_driver: Switch::builder("AV_A_Sw_Heizung_FRR", Some(CockpitSide::A))
                .event_toggle("Heizung_Fahrerstand")
                .snd_toggle("Snd_A_Switch")
                .build(),

            a_sw_parking_heater: StepSwitch::builder("AV_A_Sw_Standheizung", Some(CockpitSide::A))
                .event("Standheizung_Plus", SwitchEventAction::Plus)
                .event("Standheizung_Minus", SwitchEventAction::Minus)
                .snd_default_plus("Snd_A_Switch")
                .snd_default_minus("Snd_A_Switch")
                .max(2)
                .build(),

            a_sw_driver_seat_heater: Switch::builder("AV_A_Sw_Sitzheizung", Some(CockpitSide::A))
                .event_toggle("Sitzheizung")
                .snd_toggle("Snd_A_Switch")
                .build(),
            a_sw_mirror_heater: Switch::builder("AV_A_Sw_Spiegelheizung", Some(CockpitSide::A))
                .event_toggle("Spiegelheizung")
                .snd_toggle("Snd_A_Switch")
                .build(),
            a_btn_window_heater: PushButton::builder(
                "AV_A_Btn_Scheibenheizung",
                "FrontWindowHeaterToggle",
                Some(CockpitSide::A),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),

            a_window_heater_timer: 0.0,
            a_window_heater_relay: false,
            a_window_heater_relay_last: false,

            a_snd_window_heater_relay_on: Sound::new(
                Some("Snd_Scheibenheizungsrelais_On"),
                None,
                None,
            ),
            a_snd_window_heater_relay_off: Sound::new(
                Some("Snd_Scheibenheizungsrelais_Off"),
                None,
                None,
            ),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let battery_switch =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;

        // Read fuses
        let _fuse_front_window_heater = com.fuse.is_on("Frontscheibenheizung");
        let _fuse_side_window_heater = com.fuse.is_on("Seitenscheibenheizung");
        let _fuse_side_window_heater_control = com.fuse.is_on("SeitenscheibenheizungSteuerung");

        // Input from key events
        self.a_sw_heater_passenger.tick();
        self.a_sw_heater_driver.tick();
        self.a_sw_parking_heater.tick();
        self.a_sw_driver_seat_heater.tick();
        self.a_sw_mirror_heater.tick();
        self.a_btn_window_heater.tick();

        // Input - Signale

        // Main logic
        if self.a_btn_window_heater.is_just_pressed() {
            if self.a_window_heater_timer <= 0.0 {
                self.a_window_heater_timer = 300.0;
            } else {
                self.a_window_heater_timer = 0.0;
            }
        }

        if self.a_window_heater_timer > 0.0 {
            self.a_window_heater_timer -= delta();
        }

        self.a_window_heater_relay_last = self.a_window_heater_relay;
        self.a_window_heater_relay = self.a_window_heater_timer > 0.0 && battery_switch;

        if self.a_window_heater_relay != self.a_window_heater_relay_last {
            if self.a_window_heater_relay {
                self.a_snd_window_heater_relay_on.start();
            } else {
                self.a_snd_window_heater_relay_off.start();
            }
        }

        // Assign output

        //===============================================================
        // MMS communication
        //===============================================================

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Warmhaltebetrieb,
            self.a_sw_parking_heater.value(true) > 0,
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::FahrgastraumheizungAus,
            self.a_sw_heater_passenger.value(true),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::FahrerraumheizungAus,
            self.a_sw_heater_driver.value(true),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::ScheibenheizungA,
            self.a_window_heater_timer > 0.0,
            Some(CockpitSide::A),
        );
    }
}

impl Default for HeatingVentilation {
    fn default() -> Self {
        Self::new()
    }
}
