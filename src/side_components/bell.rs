use lotus_extra::vehicle::CockpitSide;
use pandemist_vehicle_elements::{
    api::sound::SoundWithEnd,
    elements::tech::buttons::PushButton,
    management::{communicator::Com, enums::general_enums::CabActivState},
};

use crate::general::local_values::{WslCabState, WslExtraBellTarget};

pub struct Bell {
    a_btn_bell: PushButton,

    b_btn_bell: PushButton,

    a_bell_snd: SoundWithEnd,
    b_bell_snd: SoundWithEnd,
}

impl Bell {
    pub fn new() -> Self {
        Self {
            a_btn_bell: PushButton::builder("AV_A_Btn_Klingel", "Bell1", Some(CockpitSide::A))
                .snd_press("Snd_A_BtnDn")
                .snd_release("Snd_A_BtnUp")
                .build(),
            b_btn_bell: PushButton::builder("AV_B_Btn_Klingel", "Bell1", Some(CockpitSide::B))
                .snd_press("Snd_B_BtnDn")
                .snd_release("Snd_B_BtnUp")
                .build(),

            a_bell_snd: SoundWithEnd::new("Snd_A_Klingel_Loop", "Snd_A_Klingel_End"),
            b_bell_snd: SoundWithEnd::new("Snd_B_Klingel_Loop", "Snd_B_Klingel_End"),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let cab_b_activ =
            com.lv.get_or(WslCabState(1), CabActivState::default()) > CabActivState::Off;

        let railbrake_bell = com.lv.get_or(WslExtraBellTarget, false);

        // Read fuses

        // Input from key events
        self.a_btn_bell.tick();
        let bell_a_target = self.a_btn_bell.value(cab_a_activ) || (railbrake_bell && cab_a_activ);
        self.b_btn_bell.tick();
        let bell_b_target = self.b_btn_bell.value(cab_b_activ) || (railbrake_bell && cab_b_activ);

        // Input - Signale

        // Main logic

        // Assign output
        self.a_bell_snd.tick(bell_a_target);
        self.b_bell_snd.tick(bell_b_target);
    }
}

impl Default for Bell {
    fn default() -> Self {
        Self::new()
    }
}
