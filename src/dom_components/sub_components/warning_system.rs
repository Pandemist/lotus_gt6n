use lotus_script::time::delta;
use pandemist_vehicle_elements::{
    api::{
        light::{BlinkRelais, LightBulb},
        sound::Sound,
    },
    management::{
        communicator::Com,
        enums::{door_enums::DoorTarget, state_enums::ChangedState},
    },
};

use crate::general::local_values::{WslBatteryMainSwitch, WslLowVoltageNorm, WslSpeedometerKmh};

pub struct Warnanlage {
    // Warnanlage
    warn_relais_1_out: BlinkRelais,
    warn_relais_2_out: BlinkRelais,
    warn_relais_3_out: BlinkRelais,
    warn_relais_4_out: BlinkRelais,
    warn_light_1_out: LightBulb,
    warn_light_2_out: LightBulb,
    warn_light_3_out: LightBulb,
    warn_light_4_out: LightBulb,
    warn_light_lift: LightBulb,
    snd_warning_out: Sound,
    snd_warn_relais: Sound,
    warnsignal_run_timer: f32,
    warnsignal_out_timer: f32,
    warnsignal_active_last: bool,
}

const DOORWARN_OFF_SPEED: f32 = 2.0;
const DOORWARN_OFF_TIME: f32 = 20.0;
const DOORWARN_OUT_SOUND: f32 = 2.7;

impl Warnanlage {
    pub fn new() -> Self {
        Self {
            warn_relais_1_out: BlinkRelais::new(0.393, 0.1965, 0.1965),
            warn_relais_2_out: BlinkRelais::new(0.393, 0.1965, 0.1965),
            warn_relais_3_out: BlinkRelais::new(0.393, 0.1965, 0.1965),
            warn_relais_4_out: BlinkRelais::new(0.393, 0.1965, 0.1965),
            warn_light_1_out: LightBulb::new("L_Door_1_Warnlicht_Aussen", 25.0),
            warn_light_2_out: LightBulb::new("L_Door_2_Warnlicht_Aussen", 25.0),
            warn_light_3_out: LightBulb::new("L_Door_3_Warnlicht_Aussen", 25.0),
            warn_light_4_out: LightBulb::new("L_Door_4_Warnlicht_Aussen", 25.0),
            warn_light_lift: LightBulb::new("L_A_Hubliftwarnung", 25.0),
            snd_warning_out: Sound::new(None, Some("Snd_Door_Warning_Out"), None),
            snd_warn_relais: Sound::new_simple(Some("Snd_Relais_Doorwarn")),
            warnsignal_run_timer: 0.0,
            warnsignal_out_timer: 0.0,
            warnsignal_active_last: false,
        }
    }

    pub fn tick(
        &mut self,
        door_target: DoorTarget,
        force_close_requested: bool,
        wheelchair_blinkrelais: bool,
        com: &mut Com,
    ) -> bool {
        // Read local signals
        let battery =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let km_h = com.lv.get_or(WslSpeedometerKmh, 0.0);

        // Read fuses
        let fuse_central_door_control = com.fuse.is_on("ZentraleTuersteuerung");

        // ist mindestens Freigabe feststellen
        let is_released = door_target >= DoorTarget::Release;

        // Bedingungen prüfen, ob die Warnanlage laufen soll
        if is_released || !fuse_central_door_control || !battery || (km_h > DOORWARN_OFF_SPEED) {
            self.warnsignal_run_timer = -1.0;
        } else if force_close_requested {
            self.warnsignal_run_timer = DOORWARN_OFF_TIME;
        }

        let warnanlage_aktiv = self.warnsignal_run_timer > 0.0;

        if warnanlage_aktiv {
            self.warnsignal_run_timer -= delta();
        }

        if warnanlage_aktiv && !self.warnsignal_active_last {
            self.warnsignal_out_timer = DOORWARN_OUT_SOUND;
        }

        if self.warnsignal_out_timer > 0.0 {
            self.warnsignal_out_timer -= delta();
        }

        self.snd_warning_out.update_volume(
            (self.warnsignal_out_timer > 0.0 && fuse_central_door_control) as u8 as f32,
        );

        self.warnsignal_active_last = warnanlage_aktiv;

        // Warnanlage anwenden
        self.snd_warn_relais
            .start_stop(warnanlage_aktiv && fuse_central_door_control);

        if warnanlage_aktiv {
            self.warn_relais_2_out.tick();
            self.warn_relais_3_out.tick();
            self.warn_relais_4_out.tick();
        } else {
            self.warn_relais_2_out.reset();
            self.warn_relais_3_out.reset();
            self.warn_relais_4_out.reset();
        }

        if warnanlage_aktiv {
            self.warn_relais_1_out.tick();
        } else {
            self.warn_relais_1_out.reset();
        }

        self.warn_light_1_out.tick(
            ((self.warn_relais_1_out.is_on || wheelchair_blinkrelais) as u8 as f32) * voltage,
        );
        self.warn_light_2_out
            .tick((self.warn_relais_2_out.is_on as u8 as f32) * voltage);
        self.warn_light_3_out
            .tick((self.warn_relais_3_out.is_on as u8 as f32) * voltage);
        self.warn_light_4_out
            .tick((self.warn_relais_4_out.is_on as u8 as f32) * voltage);

        self.warn_light_lift
            .tick((wheelchair_blinkrelais as u8 as f32) * voltage);

        warnanlage_aktiv
    }
}

impl Default for Warnanlage {
    fn default() -> Self {
        Self::new()
    }
}
