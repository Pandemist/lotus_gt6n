use std::rc::Rc;

use lotus_script::{prelude::Message, time::delta};
use pandemist_vehicle_elements::{
    api::{key_event::KeyEventCab, light::Light, sound::Sound},
    components::traction::railbrakes::Railbrakes,
    elements::tech::buttons::PushButton,
    management::{communicator::Com, enums::general_enums::TrainFormationSwitch},
    messages::{coupling_handler::UniversalCouplingLine, gt6n_coupling_messages::CouplerRailbrake},
};

use crate::{
    dom_components::sub_components::anti_slide_anti_skid::AntiSlipAntiSlideProtectionUnit,
    general::local_values::{
        WslCabIndicatorBrightness, WslEmergencyBrakes, WslExtraBellTarget, WslLighttest,
        WslLowVoltageNorm, WslSpeedometerKmh, WslTrainFormationSwitch,
    },
};

const RAILBRAKEFORCE: f32 = 128000.0;

const RAILBRAKE_FLASH_TIME: f32 = 1.5;
const RAILBRAKE_FLASH_TIME_ON: f32 = RAILBRAKE_FLASH_TIME / 2.0;

pub struct Railbrake {
    railbrake_coupling: UniversalCouplingLine<bool, CouplerRailbrake>,

    btn_flash_timer: f32,

    railbrake_a: Railbrakes,
    railbrake_b: Railbrakes,
    railbrake_c: Railbrakes,

    snd_railbrake_relais_on: Sound,
    snd_railbrake_relais_off: Sound,

    railbrake_relais: bool,

    a_btn_railbrake: PushButton,

    a_lm_railbrake: Light,
}

impl Railbrake {
    pub fn new() -> Self {
        Self {
            railbrake_coupling: UniversalCouplingLine::new(CouplerRailbrake {}, (true, true)),

            btn_flash_timer: 0.0,

            railbrake_a: Railbrakes::builder(0, RAILBRAKEFORCE)
                .animation("AV_Schienenbremse_A")
                .snd_railbrake_on("Snd_Mg_A")
                .snd_railbrake_friction("Snd_Mg_A_Friction_vol", Rc::new(|x| x * 1.5 - 0.5))
                .build(),
            railbrake_b: Railbrakes::builder(2, RAILBRAKEFORCE)
                .animation("AV_Schienenbremse_B")
                .snd_railbrake_on("Snd_Mg_B")
                .snd_railbrake_friction("Snd_Mg_B_Friction_vol", Rc::new(|x| x * 1.5 - 0.5))
                .build(),
            railbrake_c: Railbrakes::builder(1, RAILBRAKEFORCE)
                .animation("AV_Schienenbremse_C")
                .snd_railbrake_on("Snd_Mg_C")
                .snd_railbrake_friction("Snd_Mg_C_Friction_vol", Rc::new(|x| x * 1.5 - 0.5))
                .build(),

            snd_railbrake_relais_on: Sound::new_simple(Some("Snd_Schienenbremsrelais_On")),
            snd_railbrake_relais_off: Sound::new(Some("Snd_Schienenbremsrelais_Off"), None, None),

            railbrake_relais: false,

            a_btn_railbrake: PushButton::builder(
                "AV_A_Btn_Schienenbremse",
                "RailBrake",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),

            a_lm_railbrake: Light::new(Some("LM_A_Schienenbremse")),
        }
    }

    pub fn tick(
        &mut self,
        other_target: bool,
        anti_slip_anti_slide_unit: &AntiSlipAntiSlideProtectionUnit,
        com: &mut Com,
    ) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let km_h = com.lv.get_or(WslSpeedometerKmh, 0.0);
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let cab_indicator_light_level = com.lv.get_or(WslCabIndicatorBrightness(0), 1.0);
        let emergency_brake = com.lv.get_or(WslEmergencyBrakes, false);
        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);

        // Read fuses
        let fuse_railbrake_protector = com.fuse.is_on("Schienenbremsschuetz");

        // Input from key events
        self.a_btn_railbrake.tick();
        let railbrake_btn = self.a_btn_railbrake.value(true) && fuse_railbrake_protector;

        let railbrake_override = (km_h.abs() > 0.1) && other_target;

        // Signal durch die Zugsteuerleitung geben
        self.railbrake_coupling.update_permit(
            true,
            train_formation_switch != TrainFormationSwitch::Leading,
        );

        self.railbrake_coupling
            .update_local(railbrake_btn || railbrake_override);
        let railbraken_target = self.railbrake_coupling.get_value();

        // Input - Signale

        // Main logic
        let emergency_brake_falshing = if emergency_brake {
            self.btn_flash_timer += delta();
            if self.btn_flash_timer > RAILBRAKE_FLASH_TIME {
                self.btn_flash_timer -= RAILBRAKE_FLASH_TIME;
            }
            self.btn_flash_timer > RAILBRAKE_FLASH_TIME_ON
        } else {
            self.btn_flash_timer = 0.0;
            false
        };

        let target_override_a = false;
        let target_override_b =
            anti_slip_anti_slide_unit.anti_slide_railbrake && (km_h.abs() > 5.0);
        let target_override_c =
            anti_slip_anti_slide_unit.anti_slide_railbrake && (km_h.abs() > 25.0);

        self.railbrake_a
            .tick(railbraken_target || target_override_a, voltage, voltage);
        self.railbrake_a
            .tick(railbraken_target || target_override_b, voltage, voltage);
        self.railbrake_a
            .tick(railbraken_target || target_override_c, voltage, voltage);

        let railbrake_state =
            self.railbrake_a.state || self.railbrake_b.state || self.railbrake_c.state;

        let railbrake_klingel = railbrake_state && !anti_slip_anti_slide_unit.anti_slide_railbrake;
        com.lv.set(WslExtraBellTarget, railbrake_klingel);

        // Schienenbremsrelais
        let railbrake_relais_last = self.railbrake_relais;
        self.railbrake_relais = railbrake_state;

        // Assign output
        if railbrake_relais_last != self.railbrake_relais {
            if self.railbrake_relais {
                self.snd_railbrake_relais_on.start();
            } else {
                self.snd_railbrake_relais_off.start();
            }
        }

        self.a_lm_railbrake.set_brightness(
            voltage
                * cab_indicator_light_level
                * (light_test || railbrake_state || emergency_brake_falshing) as u8 as f32,
        );
    }

    pub fn on_message(&mut self, msg: Message) {
        self.railbrake_coupling.on_message(msg.clone());
    }
}

impl Default for Railbrake {
    fn default() -> Self {
        Self::new()
    }
}
