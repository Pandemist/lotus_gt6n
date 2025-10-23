use lotus_script::time::delta;
use pandemist_vehicle_elements::{
    api::{
        key_event::KeyEventCab, light::Light, simulation_settings::deadmans_switch, sound::Sound,
    },
    elements::tech::{buttons::PushButton, seals::SealedSwitch, switches::Switch},
    management::{communicator::Com, enums::general_enums::CabActivState},
    messages::diagnostic_messages::{DiagnosticFaultKind, DiagnosticMessageSender},
};

use crate::general::local_values::{
    WslCabIndicatorBrightness, WslCabState, WslLighttest, WslLowVoltageNorm, WslSpeedometerKmh,
};

const SIFA_TIME_S: f32 = 3.0;

pub struct Sifa {
    mms_fault_sender: DiagnosticMessageSender,

    a_pedal_sifa: PushButton,
    a_btn_throttle_lever_sifa: PushButton,
    a_btn_sifa_reset: PushButton,

    a_sw_bypass_switch_sifa_seal: SealedSwitch,

    a_lm_sifa: Light,

    sifa_alert: bool,
    snd_a_sifa_alert: Sound,

    timer: f32,
    pub forced_brake: bool,
    pub zeroing_constrain: bool,
    sifa_clear: bool,
}

impl Sifa {
    pub fn new() -> Self {
        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),

            a_pedal_sifa: PushButton::builder(
                "AV_A_Pedal_Sifa",
                "Pedal_Sifa",
                Some(KeyEventCab::ACab),
            )
            .build(),
            a_btn_throttle_lever_sifa: PushButton::builder(
                "AV_A_Sollwertgeber_Sifa",
                "HoldToRun",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_Sollwertgeber_SiFa_Dn")
            .snd_release("Snd_A_Sollwertgeber_SiFa_Up")
            .build(),

            a_btn_sifa_reset: PushButton::builder(
                "AV_A_Btn_SiFa_Reset",
                "HoldToRun_Btn",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),

            a_sw_bypass_switch_sifa_seal: SealedSwitch::new(
                Some(KeyEventCab::ACab),
                "vis_A_Plombe_Hilfsschalter_Sifa",
                "Plombe_Hilfsschalter_Sifa",
                Switch::builder("AV_A_Sw_Hilfsschalter_Sifa", Some(KeyEventCab::ACab))
                    .event_toggle("Hilfsschalter_Sifa")
                    .snd_toggle("Snd_A_Switch")
                    .build(),
            ),

            a_lm_sifa: Light::new(Some("LM_A_Sifa")),

            sifa_alert: false,
            snd_a_sifa_alert: Sound::new_simple(Some("Snd_A_Sifa_Warnung")),

            timer: 0.0,
            forced_brake: false,
            zeroing_constrain: false,
            sifa_clear: false,
        }
    }

    pub fn tick(&mut self, sifa_aktiv: bool, throttle_lever_zeroed: bool, com: &mut Com) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let cab_indicator_light_level = com.lv.get_or(WslCabIndicatorBrightness(0), 1.0);
        let km_h = com.lv.get_or(WslSpeedometerKmh, 0.0);

        // Read fuses
        let fuse_sifa = com.fuse.is_on("KWRsifaEingangssingal");

        // Input from key events
        self.a_pedal_sifa.tick();
        self.a_btn_throttle_lever_sifa.tick();
        self.a_btn_sifa_reset.tick();

        self.a_sw_bypass_switch_sifa_seal.tick();

        let veh_not_moving = km_h < 0.1;
        let local_activ = cab_a_activ
            && (self.a_pedal_sifa.value(true)
                || self.a_btn_throttle_lever_sifa.value(true)
                || deadmans_switch());

        // Input - Signale

        // Main logic
        if sifa_aktiv && fuse_sifa {
            if self.forced_brake {
                if self.a_btn_sifa_reset.value(veh_not_moving) {
                    self.sifa_clear = false;
                }
                if veh_not_moving && throttle_lever_zeroed && !self.sifa_clear {
                    self.zeroing_constrain = false;
                }
                // Zwangsbremsung erst auflösen, wenn im Stand die Sifa gelöscht und eine Nullstellung des SWG erfolgt ist
                if veh_not_moving && self.sifa_clear && self.zeroing_constrain {
                    self.forced_brake = false;
                }
            } else {
                let ok = veh_not_moving || local_activ;

                self.sifa_alert = !ok;

                if ok {
                    self.timer = SIFA_TIME_S;
                } else {
                    self.timer -= delta();
                }

                if self.timer < 0.0 {
                    self.forced_brake = !self.a_sw_bypass_switch_sifa_seal.switch.value(true);
                    self.sifa_clear = true;
                    self.zeroing_constrain = true;
                }
            }
        } else {
            self.forced_brake = false;
            self.sifa_clear = false;
            self.zeroing_constrain = false;
            self.timer = SIFA_TIME_S;
            self.sifa_alert = false;
        }

        //===============================================================
        // MMS communication
        //===============================================================

        self.mms_fault_sender.send(
            DiagnosticFaultKind::SifaUeberbrueckt,
            self.a_sw_bypass_switch_sifa_seal.switch.value(cab_a_activ),
            None,
        );

        // Assign output
        self.a_lm_sifa.set_brightness(
            voltage
                * cab_indicator_light_level
                * ((cab_a_activ && self.sifa_alert) || light_test) as u8 as f32,
        );

        self.snd_a_sifa_alert
            .start_stop(cab_a_activ && self.sifa_alert);
    }
}

impl Default for Sifa {
    fn default() -> Self {
        Self::new()
    }
}
