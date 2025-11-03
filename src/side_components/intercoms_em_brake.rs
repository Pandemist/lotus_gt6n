use lotus_extra::vehicle::CockpitSide;
use lotus_script::{prelude::Message, time::delta};
use pandemist_vehicle_elements::{
    api::{light::Light, sound::Sound},
    components::gt6n::intercom::IntercomGt6n,
    elements::tech::{
        buttons::PushButton,
        seals::SealedSwitch,
        switches::{StepSwitch, Switch, SwitchEventAction},
    },
    management::{
        communicator::Com,
        enums::{
            general_enums::{CabActivState, TrainFormationSwitch},
            state_enums::ChangedState,
        },
        structs::general_structs::TrainActivState,
    },
    messages::{
        coupling_handler::UniversalCouplingLine,
        diagnostic_messages::{DiagnosticFaultKind, DiagnosticMessageSender},
        gt6n_coupling_messages::CouplerEmergencyBrake,
    },
};

use crate::general::local_values::{
    WslBatteryMainSwitch, WslCabIndicatorBrightness, WslCabOffButStillThere, WslEmergencyBrakes,
    WslLighttest, WslLowVoltageNorm, WslTrainFormationSwitch, WslTrainState,
};

pub struct IntercomsEmBrake {
    mms_fault_sender: DiagnosticMessageSender,
    em_brake_coupling: UniversalCouplingLine<bool, CouplerEmergencyBrake>,

    a_btn_emergency_brake: PushButton,
    b_btn_emergency_shut_off: PushButton,

    a_sw_intercom: StepSwitch,
    a_sw_bypass_switch_em_brake_seal: SealedSwitch,

    emergency_brake_1: Switch,
    emergency_brake_2: Switch,
    emergency_brake_3: Switch,
    emergency_brake_4: Switch,

    intercom_1: IntercomGt6n,
    intercom_2: IntercomGt6n,
    intercom_3: IntercomGt6n,
    intercom_4: IntercomGt6n,

    current_activ: Option<usize>,
    current_activ_last: Option<usize>,
    call_queue_high: Vec<usize>,
    call_queue_low: Vec<usize>,

    a_lm_intercom: Light,
    snd_intercom_activated: Sound,

    talk_acitv_timer: f32,
    cab_off_timer: f32,
}

impl IntercomsEmBrake {
    pub fn new() -> Self {
        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),
            em_brake_coupling: UniversalCouplingLine::new(CouplerEmergencyBrake, (true, true)),

            a_btn_emergency_brake: PushButton::builder_rotate_on_release_toggle(
                "AV_A_Btn_Notbremse",
                "AV_A_Btn_Notbremse_Rot",
                "Notbremse",
                Some(CockpitSide::A),
            )
            .build(),
            b_btn_emergency_shut_off: PushButton::builder_rotate_on_release_toggle(
                "AV_B_Btn_Notbremse",
                "AV_B_Btn_Notbremse_Rot",
                "Notbremse",
                Some(CockpitSide::B),
            )
            .build(),
            a_sw_intercom: StepSwitch::builder("AV_A_Sw_Sprechstelle", Some(CockpitSide::A))
                .event("Sprechstelle_Plus", SwitchEventAction::Plus)
                .event("Sprechstelle_Minus", SwitchEventAction::Minus)
                .snd_default_plus("Snd_A_Switch")
                .snd_default_minus("Snd_A_Switch")
                .min(-1)
                .max(1)
                .min_spring()
                .max_spring()
                .build(),
            a_sw_bypass_switch_em_brake_seal: SealedSwitch::new(
                Some(CockpitSide::A),
                "vis_A_Plombe_Hilfsschalter_FG_Notbremse",
                "Plombe_Hilfsschalter_FG_Notbremse",
                Switch::builder("AV_A_Sw_Hilfsschalter_Notbremse", Some(CockpitSide::A))
                    .event_toggle("Hilfsschalter_FG_Notbremse")
                    .snd_toggle("Snd_A_Switch")
                    .build(),
            ),

            emergency_brake_1: Switch::builder("AV_Notbremse_1", None)
                .event_toggle("Notbremse_1")
                .build(),
            emergency_brake_2: Switch::builder("AV_Notbremse_2", None)
                .event_toggle("Notbremse_2")
                .build(),
            emergency_brake_3: Switch::builder("AV_Notbremse_3", None)
                .event_toggle("Notbremse_3")
                .build(),
            emergency_brake_4: Switch::builder("AV_Notbremse_4", None)
                .event_toggle("Notbremse_4")
                .build(),

            intercom_1: IntercomGt6n::new(
                1,
                "LM_Sprechstelle_1_Rot",
                "LM_Sprechstelle_1_Grn",
                "LM_Sprechstelle_1_Glb",
                "Sprechstelle_1",
                "Snd_Sprechstelle_1",
            ),
            intercom_2: IntercomGt6n::new(
                2,
                "LM_Sprechstelle_2_Rot",
                "LM_Sprechstelle_2_Grn",
                "LM_Sprechstelle_2_Glb",
                "Sprechstelle_2",
                "Snd_Sprechstelle_2",
            ),
            intercom_3: IntercomGt6n::new(
                3,
                "LM_Sprechstelle_3_Rot",
                "LM_Sprechstelle_3_Grn",
                "LM_Sprechstelle_3_Glb",
                "Sprechstelle_3",
                "Snd_Sprechstelle_3",
            ),
            intercom_4: IntercomGt6n::new(
                4,
                "LM_Sprechstelle_4_Rot",
                "LM_Sprechstelle_4_Grn",
                "LM_Sprechstelle_4_Glb",
                "Sprechstelle_4",
                "Snd_Sprechstelle_4",
            ),

            call_queue_high: vec![],
            call_queue_low: vec![],
            current_activ: None,
            current_activ_last: None,

            a_lm_intercom: Light::new(Some("LM_A_Sprechstelle")),
            snd_intercom_activated: Sound::new_simple(Some("Snd_A_Sprechstelle_Req")),

            talk_acitv_timer: 0.0,
            cab_off_timer: 0.0,
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let battery_switch =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let cab_a_activ = com
            .lv
            .get_or(WslTrainState, TrainActivState::default())
            .cab_a
            > CabActivState::Off;
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let cab_indicator_light_level = com.lv.get_or(WslCabIndicatorBrightness(0), 1.0);
        let cab_shutoff_flag = com.lv.get_or(WslCabOffButStillThere(0), false);
        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);

        // Read fuses

        // Input from key events
        self.a_sw_intercom.tick();
        if self.a_sw_intercom.just_changed_to(cab_a_activ, -1) {
            self.quit_current();
        }

        self.a_sw_bypass_switch_em_brake_seal.tick();
        let hilfsschalter_emergency_brake = self
            .a_sw_bypass_switch_em_brake_seal
            .switch
            .value(cab_a_activ);

        if self.intercom_1.pressed(cab_a_activ) {
            self.append_queue_low(1);
        }
        if self.intercom_2.pressed(cab_a_activ) {
            self.append_queue_low(2);
        }
        if self.intercom_3.pressed(cab_a_activ) {
            self.append_queue_low(3);
        }
        if self.intercom_4.pressed(cab_a_activ) {
            self.append_queue_low(4);
        }

        if self.emergency_brake_1.is_just_pressed() {
            self.append_queue_high(1);
        }
        if self.emergency_brake_2.is_just_pressed() {
            self.append_queue_high(2);
        }
        if self.emergency_brake_3.is_just_pressed() {
            self.append_queue_high(3);
        }
        if self.emergency_brake_4.is_just_pressed() {
            self.append_queue_high(4);
        }

        // Input - Signale

        // Main logic
        self.emergency_brake_1.tick();
        self.emergency_brake_2.tick();
        self.emergency_brake_3.tick();
        self.emergency_brake_4.tick();

        self.a_btn_emergency_brake.tick();
        self.b_btn_emergency_shut_off.tick();

        let some_emergency_brake = (self.emergency_brake_1.value(true)
            || self.emergency_brake_2.value(true)
            || self.emergency_brake_3.value(true)
            || self.emergency_brake_4.value(true)
            || self.a_btn_emergency_brake.value(true)
            || self.b_btn_emergency_shut_off.value(true))
            && !hilfsschalter_emergency_brake;

        // Signal durch die Zugsteuerleitung geben
        self.em_brake_coupling.update_permit(
            true,
            train_formation_switch != TrainFormationSwitch::Leading,
        );
        self.em_brake_coupling.update_local(some_emergency_brake);
        let some_emergency_brake = self.em_brake_coupling.get_value();

        com.lv.set(WslEmergencyBrakes, some_emergency_brake);

        // Notintercomn Logik
        if self.a_sw_intercom.value(!cab_a_activ) > 0 {
            self.cab_off_timer = 15.0;
        }

        if cab_a_activ || (!cab_shutoff_flag) {
            self.cab_off_timer = 0.0;
        }

        if self.cab_off_timer > 0.0 {
            self.cab_off_timer -= delta();
        }

        if self.current_activ.is_some() && cab_a_activ {
            self.talk_acitv_timer += delta();
        }

        if self.talk_acitv_timer > 300.0 {
            self.quit_current();
            self.talk_acitv_timer = 0.0;
        }

        if !battery_switch {
            self.clear_all();
        }

        self.intercom_1.tick(self.current_activ.as_ref());
        self.intercom_2.tick(self.current_activ.as_ref());
        self.intercom_3.tick(self.current_activ.as_ref());
        self.intercom_4.tick(self.current_activ.as_ref());

        // Assign output
        self.a_lm_intercom.set_brightness(
            voltage
                * cab_indicator_light_level
                * (light_test
                    || (self.current_activ.is_some()
                        || self.a_sw_intercom.value(cab_a_activ) > 0
                        || self.cab_off_timer > 0.0)) as u8 as f32,
        );

        if self.current_activ_last.is_none() && self.current_activ.is_some() && cab_a_activ {
            self.snd_intercom_activated.start();
        }
        self.current_activ_last = self.current_activ;

        //===============================================================
        // MMS communication
        //===============================================================

        self.mms_fault_sender.send(
            DiagnosticFaultKind::FGnotbremseUeberbrueckt,
            self.a_sw_bypass_switch_em_brake_seal
                .switch
                .value(cab_a_activ),
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::FahrernotbremseA,
            self.a_btn_emergency_brake.value(true),
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::FahrernotbremseB,
            self.b_btn_emergency_shut_off.value(true),
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Fahrgastnotbremse,
            self.emergency_brake_1.value(true)
                || self.emergency_brake_2.value(true)
                || self.emergency_brake_3.value(true)
                || self.emergency_brake_4.value(true),
            None,
        );
    }

    fn append_queue_high(&mut self, id: usize) {
        if !self.call_queue_high.contains(&id) {
            self.call_queue_high.push(id);
        }
        self.refresh_current();
    }

    fn append_queue_low(&mut self, id: usize) {
        if !self.call_queue_low.contains(&id) {
            self.call_queue_low.push(id);
        }
        self.refresh_current();
    }

    fn clear_all(&mut self) {
        self.call_queue_high.clear();
        self.call_queue_low.clear();
        self.refresh_current();
    }

    fn quit_current(&mut self) {
        if !self.call_queue_high.is_empty() {
            if self.call_queue_high.last().is_some()
                && !self.is_emergency_brake_acitv(*self.call_queue_high.last().unwrap())
            {
                self.call_queue_high.pop();
            }
        } else if !self.call_queue_low.is_empty()
            && self.call_queue_low.last().is_some()
            && !self.is_emergency_brake_acitv(*self.call_queue_low.last().unwrap())
        {
            self.call_queue_low.pop();
        }
        self.refresh_current();
    }

    fn refresh_current(&mut self) {
        self.current_activ = if !self.call_queue_high.is_empty() {
            self.call_queue_high.last().cloned()
        } else if !self.call_queue_low.is_empty() {
            self.call_queue_low.last().cloned()
        } else {
            None
        };
    }

    fn is_emergency_brake_acitv(&self, id: usize) -> bool {
        match id {
            1 => self.emergency_brake_1.value(true),
            2 => self.emergency_brake_2.value(true),
            3 => self.emergency_brake_3.value(true),
            4 => self.emergency_brake_4.value(true),
            _ => false,
        }
    }

    pub fn on_message(&mut self, msg: Message) {
        self.em_brake_coupling.on_message(msg.clone());
    }
}

impl Default for IntercomsEmBrake {
    fn default() -> Self {
        Self::new()
    }
}
