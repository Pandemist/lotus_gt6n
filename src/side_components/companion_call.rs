use lotus_extra::vehicle::CockpitSide;
use lotus_script::prelude::Message;
use pandemist_vehicle_elements::{
    api::{light::Light, sound::Sound},
    elements::tech::buttons::PushButton,
    management::{communicator::Com, enums::state_enums::ChangedState},
    messages::{
        coupling_handler::UniversalCouplingLine, gt6n_coupling_messages::CouplerShuntingSignal,
    },
};

use crate::general::local_values::{WslBatteryMainSwitch, WslLowVoltageNorm};

pub struct CompanionCall {
    call_coupling: UniversalCouplingLine<bool, CouplerShuntingSignal>,

    a_btn_shunting_signal: PushButton,
    b_btn_shunting_signal: PushButton,

    lm_signal: Light,

    snd_signal: Sound,

    state_last: bool,
}

impl CompanionCall {
    pub fn new() -> Self {
        let s = Self {
            call_coupling: UniversalCouplingLine::new(CouplerShuntingSignal, (true, true)),

            a_btn_shunting_signal: PushButton::builder(
                "AV_A_Btn_Rangiersignal",
                "Rangiersignal",
                Some(CockpitSide::A),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),

            b_btn_shunting_signal: PushButton::builder(
                "AV_B_Btn_Rangiersignal",
                "Rangiersignal",
                Some(CockpitSide::B),
            )
            .snd_press("Snd_B_BtnDn")
            .snd_release("Snd_B_BtnUp")
            .build(),

            lm_signal: Light::new(Some("L_Rangiersignal")),
            snd_signal: Sound::new(None, Some("Snd_Rangiersignal"), None),

            state_last: false,
        };

        s.lm_signal.set_brightness(0.0);

        s
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let battery_switch =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);

        // Read fuses

        // Input from key events
        self.a_btn_shunting_signal.tick();
        self.b_btn_shunting_signal.tick();

        let local_a = self.a_btn_shunting_signal.value(battery_switch);
        let local_b = self.b_btn_shunting_signal.value(battery_switch);

        // Input - Signale

        // Main logic
        // Wert durch die Zugsteuerleitung geben
        self.call_coupling.update_local(local_a || local_b);
        let state = self.call_coupling.get_value() && battery_switch;

        // Assign output
        self.snd_signal.update_volume(state as u8 as f32);

        if state != self.state_last {
            self.lm_signal
                .set_brightness(voltage * (state as u8 as f32));

            self.state_last = state;
        }
    }

    pub fn on_message(&mut self, _msg: Message) {}
}

impl Default for CompanionCall {
    fn default() -> Self {
        Self::new()
    }
}
