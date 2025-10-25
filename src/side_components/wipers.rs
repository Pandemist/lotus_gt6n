use std::{f32::consts::PI, rc::Rc};

use lotus_extra::vehicle::CockpitSide;
use pandemist_vehicle_elements::{
    components::general::wiper::Wiper,
    elements::tech::switches::{StepSwitch, Switch, SwitchEventAction},
    management::{
        communicator::Com,
        enums::general_enums::{CabActivState, WiperTarget},
    },
};

use crate::general::local_values::{WslCabState, WslLowVoltageNorm};

pub struct Wipers {
    a_sw_wiper: StepSwitch,
    a_sw_wiper_side: Switch,

    a_wiper: Wiper<WiperTarget>,
    a_wiper_side: Wiper<WiperTarget>,

    b_wiper: Wiper<WiperTarget>,

    b_sw_wiper: Switch,
}

impl Wipers {
    pub fn new() -> Self {
        Self {
            a_sw_wiper: StepSwitch::builder("AV_A_Sw_Scheibenwischer", Some(CockpitSide::A))
                .event("WiperPlus", SwitchEventAction::Plus)
                .event("WiperMinus", SwitchEventAction::Minus)
                .snd_default_plus("Snd_A_Switch")
                .snd_default_minus("Snd_A_Switch")
                .min(0)
                .max(3)
                .build(),
            a_sw_wiper_side: Switch::builder("AV_A_Sw_Scheibenwischer_Seite", Some(CockpitSide::A))
                .event_toggle("WiperSide_R")
                .snd_toggle("Snd_A_Switch")
                .build(),

            a_wiper: Wiper::builder("AV_A_Scheibenwischer_1")
                .main_anim_mapping(Rc::new(|x| {
                    (-4.0 * ((1.0 - (((0.5 * x) - 0.25) * 2.0 * PI).cos()) / 2.0 * 0.9).sqrt()
                        + 4.0 * (1.0 - (((0.5 * x) - 0.25) * 2.0 * PI).cos()) / 2.0 * 0.9)
                        * -1.0
                }))
                .add_secondary_anim(
                    "AV_A_Scheibenwischer_2",
                    Rc::new(|x| (1.0 - (x * 2.0 * PI).cos()) / 2.0),
                )
                .add_wiper_level(WiperTarget::Interval, 0.5, 3.0, 0)
                .add_wiper_level(WiperTarget::Normal, 0.5, 0.0, 0)
                .add_wiper_level(WiperTarget::Fast, 1.0, 0.0, 0)
                .build(),

            a_wiper_side: Wiper::builder("AV_A_Scheibenwischer_R")
                .main_anim_mapping(Rc::new(|x| 0.5 * ((x - 0.5) * 2.0 * PI).cos() + 0.5))
                .add_wiper_level(WiperTarget::Normal, 0.5, 0.0, 0)
                .build(),

            b_sw_wiper: Switch::builder("AV_B_Sw_Scheibenwischer", Some(CockpitSide::B))
                .snd_toggle("Snd_B_Switch")
                .event_toggle("WiperToggle")
                .event_plus("WiperPlus")
                .event_minus("WiperMinus")
                .build(),

            b_wiper: Wiper::builder("AV_B_Scheibenwischer_1")
                .main_anim_mapping(Rc::new(|x| {
                    (-4.0 * ((1.0 - (((0.5 * x) - 0.25) * 2.0 * PI).cos()) / 2.0 * 0.9).sqrt()
                        + 4.0 * (1.0 - (((0.5 * x) - 0.25) * 2.0 * PI).cos()) / 2.0 * 0.9)
                        * -1.0
                }))
                .add_secondary_anim(
                    "AV_B_Scheibenwischer_2",
                    Rc::new(|x| (1.0 - (x * 2.0 * PI).cos()) / 2.0),
                )
                .add_wiper_level(WiperTarget::Normal, 0.5, 0.0, 0)
                .build(),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let cab_b_activ =
            com.lv.get_or(WslCabState(1), CabActivState::default()) > CabActivState::Off;

        // Input from key events
        self.a_sw_wiper.tick();
        let a_wiper_target_main = match self.a_sw_wiper.value(cab_a_activ) {
            3 => WiperTarget::Fast,
            2 => WiperTarget::Normal,
            1 => WiperTarget::Interval,
            _ => WiperTarget::Off,
        };
        self.a_sw_wiper_side.tick();
        let a_wiper_target_side = match self.a_sw_wiper_side.value(cab_a_activ) {
            true => WiperTarget::Normal,
            false => WiperTarget::Off,
        };

        self.b_sw_wiper.tick();
        let b_wiper_target_main = match self.b_sw_wiper.value(cab_b_activ) {
            true => WiperTarget::Normal,
            false => WiperTarget::Off,
        };

        // Main logic
        self.a_wiper.tick(a_wiper_target_main, voltage);
        self.a_wiper_side.tick(a_wiper_target_side, voltage);

        self.b_wiper.tick(b_wiper_target_main, voltage);
    }
}

impl Default for Wipers {
    fn default() -> Self {
        Self::new()
    }
}
