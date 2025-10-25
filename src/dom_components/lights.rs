use lotus_extra::{messages::std::LightSender, vehicle::CockpitSide};
use lotus_script::{prelude::Message, time::delta};
use pandemist_vehicle_elements::{
    api::{
        key_event::KeyEvent,
        light::{Light, SimpleBlinker},
        sound::Sound,
    },
    elements::tech::{
        buttons::PushButton,
        key_switch::{KeyDepot, KeySwitch},
        switches::{StepSwitch, Switch, SwitchEventAction},
    },
    management::{
        communicator::Com,
        enums::{
            general_enums::{CabActivState, TrainFormationSwitch},
            state_enums::ChangedState,
            traction_enums::DirectionOfDriving,
        },
    },
    messages::{
        coupling_handler::UniversalCouplingLine,
        diagnostic_messages::{DiagnosticFaultKind, DiagnosticMessageSender},
        gt6n_coupling_messages::{CouplerIndicator, CouplerInteriorLight, Indicator},
    },
};

use crate::general::{
    local_values::{
        WslBatteryMainSwitch, WslBrakelight, WslCabIndicatorBrightness, WslCabState,
        WslConverterVoltageNorm, WslDirectionOfDriving, WslEnergencyLight, WslInteriorLight,
        WslLighttest, WslLowVoltageNorm, WslPermanentVoltageNorm, WslTrainFormationSwitch,
        WslWheelchairHelperActive,
    },
    setup::{const_veh_variant, FahrzeugVariante},
};

pub struct Lights {
    mms_fault_sender: DiagnosticMessageSender,
    light_sender: LightSender,

    a_btn_light_test: PushButton,

    a_lm_light_test: Light,

    a_sw_dimmer: StepSwitch,
    a_dimmer_level: f32,

    // Exterior light
    a_sw_exterior_lights: StepSwitch,

    l_a_parking_light: Light,
    l_a_low_beam: Light,
    l_a_high_beam: Light,
    l_a_top_light: Light,

    l_a_marker_light: Light,

    l_b_brake_light: Light,
    l_b_rear_light: Light,
    l_b_reversing_light: Light,

    a_lm_high_beam: Light,

    // Indicator
    indicator_coupling: UniversalCouplingLine<Indicator, CouplerIndicator>,

    a_sw_indicator: StepSwitch,
    a_btn_warnindicator: PushButton,

    b_sw_indicator: StepSwitch,

    indicator: SimpleBlinker,
    indicator_front: SimpleBlinker,
    indicator_rear: SimpleBlinker,

    l_indicator_rechts: Light,
    l_indicator_links: Light,

    a_lm_indicator_l: Light,
    a_lm_indicator_r: Light,
    a_lm_warnindicator: Light,

    b_lm_indicator_l: Light,
    b_lm_indicator_r: Light,

    snd_indicator_on: Sound,
    snd_indicator_off: Sound,

    indicator_on_last: bool,

    // Draivers cab light
    a_sw_driver_cab_light: StepSwitch,
    a_driver_cab_light_override: KeyEvent,

    a_tacho_light_off_timer: f32,
    a_exterior_light_last: bool,

    l_a_instrument_light: Light,

    l_a_driver_cab_light: Light,
    l_a_driver_cab_light_begleiter: Light,

    // Interior light
    interior_light_coupling: UniversalCouplingLine<bool, CouplerInteriorLight>,

    a_sw_interior_light: Switch,

    a_key_emergency_light_1: KeySwitch,
    a_key_emergency_light_2: KeySwitch,

    activ_a_cab_last: bool,
    emergency_light_timer: f32,

    l_interior_light: Light,
    l_emergency_light_lq: Light,
    l_emergency_light_tex: Light,
}

impl Lights {
    pub fn new(driver_key: KeyDepot) -> Self {
        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),
            light_sender: LightSender::default(),

            a_btn_light_test: PushButton::builder(
                "AV_A_Btn_Lampentest",
                "Lampentest",
                Some(CockpitSide::A),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),
            a_lm_light_test: Light::new(Some("LM_A_Lampentest")),

            a_sw_dimmer: StepSwitch::builder("AV_A_Sw_Dimmer", Some(CockpitSide::A))
                .event("Dimmer_Plus", SwitchEventAction::Plus)
                .event("Dimmer_Minus", SwitchEventAction::Minus)
                .snd_default_plus("Snd_A_Switch")
                .snd_default_minus("Snd_A_Switch")
                .min(-1)
                .max(1)
                .min_spring()
                .max_spring()
                .build(),

            a_dimmer_level: 1.0,

            // Aussenlicht
            a_sw_exterior_lights: StepSwitch::builder(
                "AV_A_Sw_Aussenbeleuchtung",
                Some(CockpitSide::A),
            )
            .event("FrontLightPlus", SwitchEventAction::Plus)
            .event("FrontLightMinus", SwitchEventAction::Minus)
            .event("FrontLightOff", SwitchEventAction::Set(0))
            .event("FrontLightPark", SwitchEventAction::Set(1))
            .event("FrontLightDim", SwitchEventAction::Set(2))
            .event("FrontLightFull", SwitchEventAction::Set(3))
            .snd_default_plus("Snd_A_Switch")
            .snd_default_minus("Snd_A_Switch")
            .min(0)
            .max(3)
            .build(),

            l_a_parking_light: Light::new(Some("L_A_Standlicht")),
            l_a_low_beam: Light::new(Some("L_A_Abblendlicht")),
            l_a_high_beam: Light::new(Some("L_A_Fernlicht")),
            l_a_top_light: Light::new(Some("L_A_Spitzenlicht")),

            l_a_marker_light: Light::new(Some("L_A_Begrenzungslicht")),

            l_b_brake_light: Light::new(Some("L_B_Bremslicht")),
            l_b_rear_light: Light::new(Some("L_B_Ruecklicht")),
            l_b_reversing_light: Light::new(Some("L_B_Rueckfahrlicht")),

            a_lm_high_beam: Light::new(Some("LM_A_Fernlicht")),

            // Blinker
            indicator_coupling: UniversalCouplingLine::new(CouplerIndicator {}, (true, true)),

            a_sw_indicator: StepSwitch::builder("AV_A_Sw_Blinker", Some(CockpitSide::A))
                .event("IndicatorOff", SwitchEventAction::Set(0))
                .event("IndicatorToRight", SwitchEventAction::Plus)
                .event("IndicatorToLeft", SwitchEventAction::Minus)
                .event("IndicatorToggleRight", SwitchEventAction::Set(1))
                .event("IndicatorToggleLeft", SwitchEventAction::Set(-1))
                .snd_default_plus("Snd_A_Switch")
                .snd_default_minus("Snd_A_Switch")
                .max(1)
                .min(-1)
                .build(),

            a_btn_warnindicator: PushButton::builder_hold_mode(
                "AV_A_Btn_Warnblinker",
                "IndicatorWarn",
                Some(CockpitSide::A),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),

            b_sw_indicator: StepSwitch::builder("AV_B_Sw_Blinker", Some(CockpitSide::B))
                .event("IndicatorOff", SwitchEventAction::Set(0))
                .event("IndicatorToRight", SwitchEventAction::Plus)
                .event("IndicatorToLeft", SwitchEventAction::Minus)
                .event("IndicatorToggleRight", SwitchEventAction::Set(1))
                .event("IndicatorToggleLeft", SwitchEventAction::Set(-1))
                .snd_default_plus("Snd_B_Switch")
                .snd_default_minus("Snd_B_Switch")
                .max(1)
                .min(-1)
                .build(),

            indicator: SimpleBlinker::new(0.32, 0.43),
            indicator_front: SimpleBlinker::new(0.32, 0.43),
            indicator_rear: SimpleBlinker::new(0.32, 0.43),

            l_indicator_rechts: Light::new(Some("L_Blinker_Rechts")),
            l_indicator_links: Light::new(Some("L_Blinker_Links")),

            a_lm_indicator_l: Light::new(Some("LM_A_Blinker_L")),
            a_lm_indicator_r: Light::new(Some("LM_A_Blinker_R")),
            a_lm_warnindicator: Light::new(Some("LM_A_Warnblinker")),

            b_lm_indicator_l: Light::new(Some("LM_B_Blinker_L")),
            b_lm_indicator_r: Light::new(Some("LM_B_Blinker_R")),

            snd_indicator_on: Sound::new_simple(Some("Snd_Relais_Blinker_On")),
            snd_indicator_off: Sound::new_simple(Some("Snd_Relais_Blinker_Off")),

            indicator_on_last: false,

            // Fahrerraumlicht
            a_sw_driver_cab_light: StepSwitch::builder(
                "AV_A_Sw_Fahrerraumlicht",
                Some(CockpitSide::A),
            )
            .event("CockpitLightPlus", SwitchEventAction::Plus)
            .event("CockpitLightMinus", SwitchEventAction::Minus)
            .snd_default_plus("Snd_A_Switch")
            .snd_default_minus("Snd_A_Switch")
            .max(2)
            .build(),
            a_driver_cab_light_override: KeyEvent::new(
                Some("CockpitLightToggle"),
                Some(CockpitSide::A),
            ),

            a_tacho_light_off_timer: 0.0,
            a_exterior_light_last: false,

            l_a_instrument_light: Light::new(Some("L_A_Instrumente")),
            l_a_driver_cab_light: Light::new(Some("L_A_Fahrerraumlicht")),
            l_a_driver_cab_light_begleiter: Light::new(Some("L_A_Fahrerraumlicht_Begleiter")),

            // Interiorlight
            interior_light_coupling: UniversalCouplingLine::new(CouplerInteriorLight, (true, true)),

            a_sw_interior_light: Switch::builder("AV_A_Sw_Innenbeleuchtung", Some(CockpitSide::A))
                .event_toggle("CabinLightToggle")
                .event_plus("CabinLightPlus")
                .event_minus("CabinLightMinus")
                .snd_toggle("Snd_A_Switch")
                .snd_plus("Snd_A_Switch")
                .snd_minus("Snd_A_Switch")
                .build(),

            a_key_emergency_light_1: KeySwitch::builder(
                driver_key.clone(),
                "AV_A_Key_Notlicht_1",
                "vis_A_Key_Notlicht_1",
                None,
            )
            .event_toggle("Key_Notlicht_1_Toggle")
            .event_turn("Key_Notlicht_1_Turn")
            .pullout_min()
            .build(),
            a_key_emergency_light_2: KeySwitch::builder(
                driver_key.clone(),
                "AV_A_Key_Notlicht_2",
                "vis_A_Key_Notlicht_2",
                None,
            )
            .event_toggle("Key_Notlicht_2_Toggle")
            .event_turn("Key_Notlicht_2_Turn")
            .pullout_min()
            .build(),

            activ_a_cab_last: false,
            emergency_light_timer: 0.0,

            l_interior_light: Light::new(Some("L_Innenraumlicht")),
            l_emergency_light_lq: Light::new(Some("L_Innenraumlicht_Notlicht_lq")),
            l_emergency_light_tex: Light::new(Some("L_Innenraumlicht_Notlicht_tex")),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let battery =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);

        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;

        // Input from key events
        self.a_sw_dimmer.tick();

        if self.a_sw_dimmer.just_changed_to(cab_a_activ, 1) {
            self.a_dimmer_level = (self.a_dimmer_level + 0.1).clamp(-0.5, 1.5);
        }
        if self.a_sw_dimmer.just_changed_to(cab_a_activ, -1) {
            self.a_dimmer_level = (self.a_dimmer_level - 0.1).clamp(-0.5, 1.5);
        }
        if const_veh_variant() == FahrzeugVariante::Gt6u {
            com.lv
                .set(WslCabIndicatorBrightness(0), self.a_dimmer_level);
        }

        self.a_btn_light_test.tick();
        let light_test = self.a_btn_light_test.is_pressed() && battery && cab_a_activ;

        // Main logic
        com.lv.set(WslLighttest(0), light_test);

        // Assign output
        self.a_lm_light_test
            .set_brightness(voltage * light_test as u8 as f32);

        let innenlicht = self.interior_light_coupling.get_value() && battery;

        //===============================================================
        // Exterior Light
        //===============================================================

        self.exterior_light(com);

        //===============================================================
        // Blinker
        //===============================================================

        self.blinker(com);

        //===============================================================
        // Cabin Light
        //===============================================================

        self.cabin_light(com);

        //===============================================================
        // Interior Light
        //===============================================================

        self.interior_light(com);

        //===============================================================
        // MMS communication
        //===============================================================

        self.mms_fault_sender.send(
            DiagnosticFaultKind::AussenbeleuchtungAeinschalten,
            cab_a_activ && innenlicht && self.a_sw_exterior_lights.value(true) <= 1,
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::AussenbeleuchtungAusgefallen,
            !(com.fuse.is_on("NahFernlicht") && com.fuse.is_on("Beleuchtungssteuerung")),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::AusfallBremslichtA,
            !(com.fuse.is_on("Begrenzungslicht")),
            Some(CockpitSide::A),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::BlinkerAusfall,
            !(com.fuse.is_on("BlinkerLinks") && com.fuse.is_on("BlinkerRechts")),
            None,
        );
    }

    fn exterior_light(&mut self, com: &mut Com) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);

        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let cab_a_runmode =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Star;

        let b_brake_light = com.lv.get_or(WslBrakelight(1), false);
        let direction_of_driving = com
            .lv
            .get_or(WslDirectionOfDriving, DirectionOfDriving::default());

        let light_test = com.lv.get_or(WslLighttest(0), false);

        // Read fuses
        let fuse_zugbeleuchtung = com.fuse.is_on("Zugbeleuchtung");
        let fuse_begrenzungslicht = com.fuse.is_on("Begrenzungslicht");
        let fuse_nah_fernlicht = com.fuse.is_on("NahFernlicht");

        // Input from key events
        self.a_sw_exterior_lights.tick();
        let a_exterior_light = self.a_sw_exterior_lights.value(true);

        // Input - Signale

        // Main logic
        self.light_sender
            .send(voltage * (a_exterior_light > 0) as u8 as f32);

        let a_parking_light =
            (a_exterior_light > 0 || cab_a_runmode) && fuse_nah_fernlicht && fuse_zugbeleuchtung;
        let a_low_beam =
            (a_exterior_light > 1 || cab_a_runmode) && fuse_nah_fernlicht && fuse_zugbeleuchtung;
        let a_high_beam = a_exterior_light > 2 && fuse_nah_fernlicht && fuse_zugbeleuchtung;

        let a_marker_light = a_exterior_light > 1 && fuse_nah_fernlicht && fuse_begrenzungslicht;

        let b_brake_light = b_brake_light && fuse_zugbeleuchtung;
        let b_reversing_light =
            a_exterior_light > 1 && direction_of_driving.backward && fuse_zugbeleuchtung;

        // Assign output
        self.l_a_parking_light
            .set_brightness(voltage * (a_parking_light) as u8 as f32);
        self.l_a_low_beam
            .set_brightness(voltage * (a_low_beam) as u8 as f32);
        self.l_a_high_beam
            .set_brightness(voltage * (a_high_beam) as u8 as f32);
        self.l_a_top_light
            .set_brightness(voltage * (a_low_beam) as u8 as f32);
        self.l_a_marker_light
            .set_brightness(voltage * (a_marker_light) as u8 as f32);

        self.l_b_brake_light
            .set_brightness(voltage * (b_brake_light) as u8 as f32);
        self.l_b_rear_light
            .set_brightness(voltage * (a_low_beam) as u8 as f32);
        self.l_b_reversing_light
            .set_brightness(voltage * (b_reversing_light) as u8 as f32);

        self.a_lm_high_beam
            .set_brightness(voltage * (light_test || (a_high_beam && cab_a_activ)) as u8 as f32);
    }

    fn blinker(&mut self, com: &mut Com) {
        // Read local signals
        let battery =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let permanent_voltage = com.lv.get_or(WslPermanentVoltageNorm, 0.0);

        let wheelchair_helper_active = com.lv.get_or(WslWheelchairHelperActive(0), false);

        let light_test = com.lv.get_or(WslLighttest(0), false);

        // Read fuses
        let fuse_indicator_r = com.fuse.is_on("BlinkerRechts");
        let fuse_indicator_l = com.fuse.is_on("BlinkerLinks");

        // Input from key events
        self.a_sw_indicator.tick();
        let a_indicator_target = self.a_sw_indicator.value(true);
        self.a_btn_warnindicator.tick();
        let a_warnindicator_target =
            self.a_btn_warnindicator.value(true) || wheelchair_helper_active;

        self.b_sw_indicator.tick();
        let b_indicator_target = self.b_sw_indicator.value(true);

        // Input - Signale

        // Main logic

        // Lokale Werte Bestimmen
        let indicator_target_a_r = a_indicator_target > 0 && battery && fuse_indicator_r;
        let indicator_target_a_l = a_indicator_target < 0 && battery && fuse_indicator_l;
        let indicator_target_a_warn = a_warnindicator_target;

        let indicator_target_b_r = b_indicator_target > 0 && battery && fuse_indicator_r;
        let indicator_target_b_l = b_indicator_target < 0 && battery && fuse_indicator_l;
        let indicator_target_b_warn = false;

        let indicator_target_r = indicator_target_a_r || indicator_target_b_l;
        let indicator_target_l = indicator_target_a_l || indicator_target_b_r;
        let indicator_target_warn = indicator_target_a_warn || indicator_target_b_warn;

        self.indicator_coupling.update_local(Indicator::new(
            indicator_target_l,
            indicator_target_r,
            indicator_target_warn,
        ));

        // Local Targets
        self.indicator.target = indicator_target_r || indicator_target_l || indicator_target_warn;

        // Front Target
        self.indicator_front.target = self.indicator_coupling.get_front().is_one();

        // Rear Target
        self.indicator_rear.target = self.indicator_coupling.get_rear().is_one();

        //set_var(
        //    "AA_Blinker_front",
        //    format!("{:?}", self.indicator_coupling.get_front()),
        //);
        //set_var(
        //    "AA_Blinker_rear",
        //    format!("{:?}", self.indicator_coupling.get_rear()),
        //);
        //set_var(
        //    "AA_Blinker_local",
        //    format!("{:?}", self.indicator_coupling.local_value),
        //);
        //set_var(
        //    "AA_Blinker_val",
        //    format!("{:?}", self.indicator_coupling.get_value()),
        //);
        //set_var(
        //    "AA_Blinker_dbg",
        //    format!("{:?}", self.indicator_coupling.dbg),
        //);

        self.indicator.tick();
        self.indicator_front.tick();
        self.indicator_rear.tick();

        let indicator_on_r = (self.indicator.lighted
            && (indicator_target_r || indicator_target_warn))
            || (self.indicator_front.lighted
                && (self.indicator_coupling.get_front().right
                    || self.indicator_coupling.get_front().warn))
            || (self.indicator_rear.lighted
                && (self.indicator_coupling.get_rear().right
                    || self.indicator_coupling.get_rear().warn));
        let indicator_on_l = (self.indicator.lighted
            && (indicator_target_l || indicator_target_warn))
            || (self.indicator_front.lighted
                && (self.indicator_coupling.get_front().left
                    || self.indicator_coupling.get_front().warn))
            || (self.indicator_rear.lighted
                && (self.indicator_coupling.get_rear().left
                    || self.indicator_coupling.get_rear().warn));
        let indicator_on_warn = (self.indicator.lighted && indicator_target_warn)
            || (self.indicator_front.lighted && self.indicator_coupling.get_front().warn)
            || (self.indicator_rear.lighted && self.indicator_coupling.get_rear().warn);

        self.l_indicator_links
            .set_brightness(indicator_on_l as u8 as f32 * permanent_voltage);
        self.l_indicator_rechts
            .set_brightness(indicator_on_r as u8 as f32 * permanent_voltage);

        self.a_lm_indicator_l.set_brightness(
            voltage * self.a_dimmer_level * (indicator_on_l || light_test) as u8 as f32,
        );
        self.a_lm_indicator_r.set_brightness(
            voltage * self.a_dimmer_level * (indicator_on_r || light_test) as u8 as f32,
        );
        self.a_lm_warnindicator.set_brightness(
            voltage * self.a_dimmer_level * (indicator_on_warn || light_test) as u8 as f32,
        );

        self.b_lm_indicator_l
            .set_brightness(voltage * (indicator_on_l) as u8 as f32);
        self.b_lm_indicator_r
            .set_brightness(voltage * (indicator_on_r) as u8 as f32);

        // Blinker Sound
        if self.indicator_on_last != self.indicator.lighted {
            if self.indicator.lighted {
                self.snd_indicator_on.start();
            } else {
                self.snd_indicator_off.start();
            }
        }

        self.indicator_on_last = self.indicator.lighted;
    }

    fn cabin_light(&mut self, com: &mut Com) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);

        // Input from key events
        if self.a_driver_cab_light_override.is_just_pressed() {
            if self.a_sw_driver_cab_light.value(true) != 0 {
                self.a_sw_driver_cab_light.set(0);
            } else {
                self.a_sw_driver_cab_light.set(2);
            }
        }

        self.a_sw_driver_cab_light.tick();
        let a_inst_light_target = self.a_sw_driver_cab_light.value(true);

        let a_exterior_light = self.a_sw_exterior_lights.value(true) > 0;

        // Input - Signale

        // Main logic
        let a_instrument_light = if const_veh_variant() == FahrzeugVariante::Gt6u {
            if (self.a_exterior_light_last && !a_exterior_light)
                || (!self.a_exterior_light_last && a_exterior_light)
            {
                self.a_tacho_light_off_timer = 0.075;
            }

            if self.a_tacho_light_off_timer > 0.0 {
                self.a_tacho_light_off_timer -= delta();
            }

            a_exterior_light && self.a_tacho_light_off_timer <= 0.0
        } else {
            a_exterior_light
        };

        self.a_exterior_light_last = a_exterior_light;

        // Assign output
        self.l_a_instrument_light
            .set_brightness(voltage * (a_instrument_light) as u8 as f32);
        self.l_a_driver_cab_light
            .set_brightness(voltage * (a_inst_light_target > 1) as u8 as f32);
        self.l_a_driver_cab_light_begleiter
            .set_brightness(voltage * (a_inst_light_target > 0) as u8 as f32);
    }

    fn interior_light(&mut self, com: &mut Com) {
        // Read local signals
        let battery =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let converter_voltage = com.lv.get_or(WslConverterVoltageNorm, 0.0);
        let direction_of_driving = com
            .lv
            .get_or(WslDirectionOfDriving, DirectionOfDriving::default());

        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;

        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);

        // Input from key events
        self.a_sw_interior_light.tick();
        let a_sw_innenbel = self.a_sw_interior_light.value(true);

        self.a_key_emergency_light_1.tick();
        let a_emergency_light_1 = self.a_key_emergency_light_1.value(true);
        self.a_key_emergency_light_2.tick();
        let a_emergency_light_2 = self.a_key_emergency_light_2.value(true);

        // Input - Signale

        // Main logic
        // Nur einschalten, wenn das der letzte Fst im Zug war der aus geht
        if !cab_a_activ && self.activ_a_cab_last && direction_of_driving.is_none() {
            self.emergency_light_timer = 30.0;
        }

        // Wert einmal durch die Zugsteuerleitung geben
        self.interior_light_coupling.update_permit(
            true,
            train_formation_switch != TrainFormationSwitch::Leading,
        );
        self.interior_light_coupling.update_local(a_sw_innenbel);
        let innenlicht = self.interior_light_coupling.get_value() && battery;

        if self.emergency_light_timer > 0.0 && innenlicht {
            self.emergency_light_timer = 0.0;
        }

        let emergency_light =
            self.emergency_light_timer > 0.0 || a_emergency_light_1 > 0 || a_emergency_light_2 > 0;

        com.lv.set(WslInteriorLight, innenlicht);
        com.lv.set(WslEnergencyLight, emergency_light);

        let innenlicht_target =
            (innenlicht || self.interior_light_coupling.get_front()) as u8 as f32;
        let emergency_light_lq_target =
            (emergency_light || innenlicht && converter_voltage < 0.5) as u8 as f32;
        let emergency_light_tex_target = (innenlicht || emergency_light) as u8 as f32;

        self.activ_a_cab_last = cab_a_activ;
        // Assign output
        self.l_interior_light
            .set_brightness(converter_voltage * innenlicht_target);
        self.l_emergency_light_lq
            .set_brightness(voltage * emergency_light_lq_target);
        self.l_emergency_light_tex
            .set_brightness(voltage * emergency_light_tex_target);
    }

    pub fn on_message(&mut self, msg: Message) {
        self.indicator_coupling.on_message(msg.clone());
        self.interior_light_coupling.on_message(msg.clone());
    }
}
