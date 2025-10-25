use lotus_extra::vehicle::CockpitSide;
use lotus_script::math::Vec2;
use pandemist_vehicle_elements::{
    api::{
        mock_enums::VehicleInitState,
        simulation_settings::{init_pos_in_train, init_ready_state},
    },
    components::general::mirror::OutsideMirror,
    elements::tech::handpin::HandPin,
    management::{communicator::Com, enums::general_enums::CabActivState},
};

use crate::general::local_values::{WslCabState, WslLowVoltageNorm};

pub struct Mirrors {
    a_sw_mirror_adjustments: HandPin,
    a_mirror_right: OutsideMirror,
}

impl Mirrors {
    pub fn new() -> Self {
        Self {
            a_sw_mirror_adjustments: HandPin::builder(
                "AV_A_Sw_SpiegelPin_X",
                "AV_A_Sw_SpiegelPin_Y",
                Some(CockpitSide::A),
            )
            .event_grab("SpiegelPin")
            .mouse_factor(0.25)
            .build(),

            a_mirror_right: OutsideMirror::builder(
                "AV_A_Spiegel_x",
                "AV_A_Spiegel_y",
                Some(CockpitSide::A),
            )
            .add_mirror_arm("AV_A_Spiegelarm")
            .keyevent_arm("MirrorRight")
            .mouse_factor_arm(-1.0 / 200.0)
            .init_arm(
                init_ready_state() > VehicleInitState::ColdAndDark && init_pos_in_train() == 0,
            )
            .mirror_movement_border(Vec2 { x: -0.2, y: -0.2 }, Vec2 { x: 0.2, y: 0.3 })
            .mirror_movement_variance(Vec2 { x: -0.05, y: -0.05 }, Vec2 { x: 0.05, y: 0.05 })
            .mirror_speed(Vec2 { x: 0.1, y: 0.1 })
            .snd_move("Snd_A_Spiegel_move")
            .snd_move_end("Snd_A_Spiegel_end")
            .build(),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;

        // Read fuses

        // Input from key events
        self.a_sw_mirror_adjustments.tick();
        let mirror_target = self.a_sw_mirror_adjustments.direction.and(cab_a_activ);

        // Input - Signale

        // Main logic

        self.a_mirror_right.mirror_target = mirror_target;
        self.a_mirror_right.tick(voltage);

        // Assign output
    }
}

impl Default for Mirrors {
    fn default() -> Self {
        Self::new()
    }
}
