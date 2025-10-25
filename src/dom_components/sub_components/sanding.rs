use lotus_extra::vehicle::CockpitSide;
use lotus_script::{prelude::Message, time::delta};
use pandemist_vehicle_elements::{
    api::{
        animation::Animation, axis::ApiRailAxis, sound::SoundWithStartAndEnd,
        visible_flag::Visiblility,
    },
    elements::{std::delay::Delay, tech::buttons::PushButton},
    management::{
        communicator::Com,
        enums::general_enums::{CabActivState, TrainFormationSwitch},
    },
    messages::{coupling_handler::UniversalCouplingLine, gt6n_coupling_messages::CouplerSanding},
};

use crate::general::local_values::{
    WslCabState, WslLowVoltageNorm, WslSpeedometerKmh, WslTractionTarget, WslTrainFormationSwitch,
};

const MAX_SANDING_TIME_S: f32 = 10.0;

pub struct Sanding {
    sanding_coupling: UniversalCouplingLine<bool, CouplerSanding>,

    sanding_axis_a: ApiRailAxis,
    sanding_axis_b: ApiRailAxis,
    sanding_axis_c: ApiRailAxis,

    sandhill_pos: f32,
    sandhill_anim: Animation,
    sandhill_n: Visiblility,
    sandhill_s: Visiblility,
    sandhill_k: Visiblility,
    sandhill_m: Visiblility,

    delay: Delay<bool>,

    a_btn_sanding: PushButton,

    snd_sanding: SoundWithStartAndEnd,

    sanding_timer: f32,
    sanding_lock_flag: bool,
}

impl Sanding {
    pub fn new() -> Self {
        Self {
            sanding_coupling: UniversalCouplingLine::new(CouplerSanding {}, (true, true)),

            sanding_axis_a: ApiRailAxis::new(1, 0),
            sanding_axis_b: ApiRailAxis::new(1, 1),
            sanding_axis_c: ApiRailAxis::new(0, 2),

            sandhill_pos: 0.0,
            sandhill_anim: Animation::new(Some("SandingHillHeight")),
            sandhill_n: Visiblility::new("SandingHillVis"),
            sandhill_s: Visiblility::new("SandingHillVis_S"),
            sandhill_k: Visiblility::new("SandingHillVis_K"),
            sandhill_m: Visiblility::new("SandingHillVis_M"),

            delay: Delay::new(1.0, false),

            a_btn_sanding: PushButton::builder("AV_A_Btn_Sanden", "Sanding", Some(CockpitSide::A))
                .snd_press("Snd_A_BtnDn")
                .snd_release("Snd_A_BtnUp")
                .build(),

            snd_sanding: SoundWithStartAndEnd::new(
                "Snd_Sand_Start",
                "Snd_Sand_Loop",
                "Snd_Sand_Stop",
            ),

            sanding_timer: 0.0,
            sanding_lock_flag: false,
        }
    }

    pub fn tick(&mut self, sanding_override: bool, com: &mut Com) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let cab_a_runmode =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Star;
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let v_kmh = com.lv.get_or(WslSpeedometerKmh, 0.0);
        let sollwert_sign = com.lv.get_or(WslTractionTarget, 0.0).signum();
        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);

        // Read fuses

        // Input from key events
        self.a_btn_sanding.tick();

        let sanding_hand = self.a_btn_sanding.value(cab_a_activ)
            && (sollwert_sign < 0.0 || (sollwert_sign > 0.0 && (v_kmh.abs() <= 10.0)));

        self.sanding_lock_flag = self.sanding_lock_flag
            || (self.a_btn_sanding.value(cab_a_runmode) && !sanding_hand)
                && self.a_btn_sanding.value(cab_a_runmode);

        let sanding_hand = sanding_hand && !self.sanding_lock_flag;

        // Input - Signale

        // Main logic
        if sanding_hand {
            self.sanding_timer -= delta();
        } else {
            self.sanding_timer = MAX_SANDING_TIME_S;
        }

        let target = sanding_hand && self.sanding_timer > 0.0;

        // Signal durch die Zugsteuerleitung geben
        self.sanding_coupling.update_permit(
            true,
            train_formation_switch != TrainFormationSwitch::Leading,
        );

        self.sanding_coupling.update_local(target);
        let target = self.sanding_coupling.get_value();

        let target = target || sanding_override;
        let target = target && voltage > 0.6;

        self.delay.tick(target);

        self.snd_sanding.tick(target);

        // Assign output
        let output_target = self.delay.output;

        self.sandhill_pos = ((self.sandhill_pos + delta() / MAX_SANDING_TIME_S).min(1.0))
            * (output_target as u8 as f32);

        self.sandhill_anim.set(self.sandhill_pos);
        self.sandhill_n.set_visbility(output_target);
        self.sandhill_s.set_visbility(output_target);
        self.sandhill_k.set_visbility(output_target);
        self.sandhill_m.set_visbility(output_target);

        self.sanding_axis_a.set_sanding(output_target);
        self.sanding_axis_b.set_sanding(output_target);
        self.sanding_axis_c.set_sanding(output_target);
    }

    pub fn on_message(&mut self, msg: Message) {
        self.sanding_coupling.on_message(msg.clone());
    }
}

impl Default for Sanding {
    fn default() -> Self {
        Self::new()
    }
}
