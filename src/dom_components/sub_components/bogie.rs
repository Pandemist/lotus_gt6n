use lotus_script::{
    math::exponential_approach, prelude::Message, time::delta, vehicle::RailQuality,
};
use pandemist_vehicle_elements::{
    api::{
        axis::ApiRailAxis,
        key_event::KeyEventCab,
        light::Light,
        sound::{Sound, SoundTarget},
    },
    elements::{std::delay::Delay, tech::buttons::PushButton},
    management::{
        communicator::Com,
        enums::{
            general_enums::{CabActivState, TrainFormationSwitch},
            traction_enums::DirectionOfDriving,
        },
    },
    messages::{
        coupling_handler::UniversalCouplingLine,
        diagnostic_messages::{DiagnosticFaultKind, DiagnosticMessageSender},
        gt6n_coupling_messages::CouplerSpringBrake,
    },
};

use crate::{
    dom_components::sub_components::anti_slide_anti_skid::AntiSlipAntiSlideProtectionUnit,
    general::local_values::{
        WslCabIndicatorBrightness, WslCabState, WslDirectionOfDriving, WslEmergencyBrakes,
        WslLighttest, WslLowVoltageNorm, WslSpeedometerKmh, WslTractionVoltageNorm,
        WslTrainFormationSwitch,
    },
};

const TRACTIONFORCE_N_ZERO: f32 = 16000.0;
const TRACTIONFORCE_N_60: f32 = 2000.0;
const MAX_BRAKEFORCE_N: f32 = 16000.0;
const E_BRAKE_LIMIT: f32 = 5.0 / 3.6;

pub struct Bogie {
    mms_fault_sender: DiagnosticMessageSender,

    spring_brake: SpringBrake,

    pub motor_a: BogieGt6n,
    pub motor_c: BogieGt6n,
    pub motor_b: BogieGt6n,

    a_btn_motor_group_a: PushButton,
    a_btn_motor_group_b: PushButton,
    a_btn_motor_group_c: PushButton,

    a_lm_motor_a: Light,
    a_lm_motor_b: Light,
    a_lm_motor_c: Light,
}

impl Bogie {
    pub fn new() -> Self {
        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),

            spring_brake: SpringBrake::default(),

            motor_a: BogieGt6n::new(
                0,
                1,
                0,
                "Snd_BrakeFlirr_A",
                "Snd_Fiep_hoch_A",
                "Snd_Fiep_tief_A",
                "Snd_Fiep_acc_A",
                "Snd_Traction_A",
            ),
            motor_c: BogieGt6n::new(
                1,
                1,
                0,
                "Snd_BrakeFlirr_C",
                "Snd_Fiep_hoch_C",
                "Snd_Fiep_tief_C",
                "Snd_Fiep_acc_C",
                "Snd_Traction_C",
            ),
            motor_b: BogieGt6n::new(
                2,
                0,
                1,
                "Snd_BrakeFlirr_B",
                "Snd_Fiep_hoch_B",
                "Snd_Fiep_tief_B",
                "Snd_Fiep_acc_B",
                "Snd_Traction_B",
            ),

            a_btn_motor_group_a: PushButton::builder_hold_mode(
                "AV_A_Btn_motor_group_A",
                "motor_group_A",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),
            a_btn_motor_group_c: PushButton::builder_hold_mode(
                "AV_A_Btn_motor_group_C",
                "motor_group_C",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),
            a_btn_motor_group_b: PushButton::builder_hold_mode(
                "AV_A_Btn_motor_group_B",
                "motor_group_B",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),

            a_lm_motor_a: Light::new(Some("LM_A_Motor_A")),
            a_lm_motor_c: Light::new(Some("LM_A_Motor_C")),
            a_lm_motor_b: Light::new(Some("LM_A_Motor_B")),
        }
    }

    pub fn tick(
        &mut self,
        traction_target: f32,
        fast_brake: bool,
        fast_and_em_brakes: bool,
        anti_slip_anti_slide_unit: &AntiSlipAntiSlideProtectionUnit,
        v_max_mps: f32,
        com: &mut Com,
    ) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let cab_indicator_light_level = com.lv.get_or(WslCabIndicatorBrightness(0), 1.0);
        let v_kmh = com.lv.get_or(WslSpeedometerKmh, 0.0);

        // Input from key events
        self.a_btn_motor_group_a.tick();
        self.a_btn_motor_group_c.tick();
        self.a_btn_motor_group_b.tick();

        // Main logic

        // Spring brake
        let spring_brake_target = self.spring_brake.tick(
            fast_and_em_brakes,
            &mut self.mms_fault_sender,
            &mut self.motor_a,
            &mut self.motor_c,
            &mut self.motor_b,
            com,
        );

        // Motor
        let traction_target =
            if v_kmh.abs() > (v_max_mps * 3.6) || anti_slip_anti_slide_unit.anti_slip_active {
                traction_target.min(0.0)
            } else {
                traction_target
            };

        let loss_of_power = (TRACTIONFORCE_N_ZERO - TRACTIONFORCE_N_60) / v_max_mps;

        self.motor_a.tick(
            traction_target * (!self.a_btn_motor_group_a.value(true)) as u8 as f32,
            fast_brake,
            anti_slip_anti_slide_unit.anti_slide_active,
            loss_of_power,
            spring_brake_target,
            com,
        );
        self.motor_c.tick(
            traction_target * (!self.a_btn_motor_group_c.value(true)) as u8 as f32,
            fast_brake,
            anti_slip_anti_slide_unit.anti_slide_active,
            loss_of_power,
            spring_brake_target,
            com,
        );
        self.motor_b.tick(
            traction_target * (!self.a_btn_motor_group_b.value(true)) as u8 as f32,
            fast_brake,
            anti_slip_anti_slide_unit.anti_slide_active,
            loss_of_power,
            spring_brake_target,
            com,
        );

        // Assign output
        self.a_lm_motor_a.set_brightness(
            voltage
                * cab_indicator_light_level
                * (light_test || self.a_btn_motor_group_a.value(cab_a_activ)) as u8 as f32,
        );
        self.a_lm_motor_c.set_brightness(
            voltage
                * cab_indicator_light_level
                * (light_test || self.a_btn_motor_group_c.value(cab_a_activ)) as u8 as f32,
        );
        self.a_lm_motor_b.set_brightness(
            voltage
                * cab_indicator_light_level
                * (light_test || self.a_btn_motor_group_b.value(cab_a_activ)) as u8 as f32,
        );

        //===============================================================
        // MMS communication
        //===============================================================

        self.mms_fault_sender.send(
            DiagnosticFaultKind::AntriebAausgegruppiert,
            self.a_btn_motor_group_a.value(cab_a_activ),
            None,
        );
        self.mms_fault_sender.send(
            DiagnosticFaultKind::AntriebBausgegruppiert,
            self.a_btn_motor_group_b.value(cab_a_activ),
            None,
        );
        self.mms_fault_sender.send(
            DiagnosticFaultKind::AntriebCausgegruppiert,
            self.a_btn_motor_group_c.value(cab_a_activ),
            None,
        );

        // spring_brake
    }

    pub fn on_message(&mut self, msg: Message) {
        self.spring_brake.on_message(msg);
    }
}

impl Default for Bogie {
    fn default() -> Self {
        Self::new()
    }
}

//====================================================================

pub struct SpringBrake {
    spring_brake_coupling: UniversalCouplingLine<bool, CouplerSpringBrake>,

    a_btn_spring_brake: PushButton,

    a_btn_spring_brake_manual_release_a: PushButton,
    a_btn_spring_brake_manual_release_b: PushButton,
    a_btn_spring_brake_manual_release_c: PushButton,

    lm_spring_brake_blinker: f32,
    a_lm_spring_brake: Light,

    a_lm_spring_brake_a: Light,
    a_lm_spring_brake_b: Light,
    a_lm_spring_brake_c: Light,
}

impl SpringBrake {
    pub fn new() -> Self {
        Self {
            spring_brake_coupling: UniversalCouplingLine::new(CouplerSpringBrake {}, (true, true)),

            a_btn_spring_brake: PushButton::builder_hold_mode(
                "AV_A_Btn_Federspeicher",
                "Federspeicher",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .init_pressed(true)
            .build(),

            a_btn_spring_brake_manual_release_a: PushButton::builder_hold_mode(
                "AV_A_Btn_FspNotloesen_A",
                "Federspeichernotloesen_A",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),
            a_btn_spring_brake_manual_release_c: PushButton::builder_hold_mode(
                "AV_A_Btn_FspNotloesen_C",
                "Federspeichernotloesen_C",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),
            a_btn_spring_brake_manual_release_b: PushButton::builder_hold_mode(
                "AV_A_Btn_FspNotloesen_B",
                "Federspeichernotloesen_B",
                Some(KeyEventCab::ACab),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),

            lm_spring_brake_blinker: 0.0,
            a_lm_spring_brake: Light::new(Some("LM_A_Federspeicher")),

            a_lm_spring_brake_a: Light::new(Some("LM_A_FspNotloesen_A")),
            a_lm_spring_brake_c: Light::new(Some("LM_A_FspNotloesen_C")),
            a_lm_spring_brake_b: Light::new(Some("LM_A_FspNotloesen_B")),
        }
    }
    pub fn tick(
        &mut self,
        fast_and_em_brakes: bool,
        mms_fault_sender: &mut DiagnosticMessageSender,
        bogie_a: &mut BogieGt6n,
        bogie_c: &mut BogieGt6n,
        bogie_b: &mut BogieGt6n,
        com: &mut Com,
    ) -> bool {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let cab_a_runmode =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Star;
        //let cab_a_activ =
        //    com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        //let cab_b_activ =
        //    com.lv.get_or(WslCabState(1), CabActivState::default()) > CabActivState::Off;
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);
        let direction_of_driving = com
            .lv
            .get_or(WslDirectionOfDriving, DirectionOfDriving::default());

        // Read fuses

        // Input from key events
        self.a_btn_spring_brake.tick();

        // Signal durch die Zugsteuerleitung geben
        self.spring_brake_coupling.update_permit(
            true,
            train_formation_switch != TrainFormationSwitch::Leading,
        );

        self.spring_brake_coupling
            .update_local(self.a_btn_spring_brake.value(true));
        let fsp_hand = self.spring_brake_coupling.get_value();

        self.a_btn_spring_brake_manual_release_a.tick();
        self.a_btn_spring_brake_manual_release_c.tick();
        self.a_btn_spring_brake_manual_release_b.tick();

        let fsp_car_offline = direction_of_driving.is_none();
        // !(cab_a_activ || cab_b_activ);
        let fsp_override = cab_a_runmode && !fsp_hand;

        // Main logic

        bogie_a.spring_brake.outgrouped = self.a_btn_spring_brake_manual_release_a.value(true);
        bogie_c.spring_brake.outgrouped = self.a_btn_spring_brake_manual_release_c.value(true);
        bogie_b.spring_brake.outgrouped = self.a_btn_spring_brake_manual_release_b.value(true);

        if fast_and_em_brakes {
            self.a_btn_spring_brake_manual_release_a.value = false;
            self.a_btn_spring_brake_manual_release_c.value = false;
            self.a_btn_spring_brake_manual_release_b.value = false;

            bogie_a.spring_brake.outgrouped = false;
            bogie_c.spring_brake.outgrouped = false;
            bogie_b.spring_brake.outgrouped = false;
        }

        self.lm_spring_brake_blinker += delta();
        if self.lm_spring_brake_blinker > 3.0 {
            self.lm_spring_brake_blinker = -3.0;
        }

        let fsp_blinker_on = self.lm_spring_brake_blinker > 1.5;

        let spring_brake_stoerung = ((bogie_a.spring_brake.broken as u8 as f32)
            + (bogie_c.spring_brake.broken as u8 as f32)
            + (bogie_b.spring_brake.broken as u8 as f32))
            as i32;

        let spring_brake_state = bogie_a.spring_brake.state
            || bogie_c.spring_brake.state
            || bogie_b.spring_brake.state
            || spring_brake_stoerung == 3
            || (fsp_blinker_on && ((spring_brake_stoerung == 1) || (spring_brake_stoerung == 2)));

        // Assign output
        self.a_lm_spring_brake
            .set_brightness(voltage * (light_test || spring_brake_state) as u8 as f32);

        self.a_lm_spring_brake_a
            .set_brightness(voltage * (light_test || bogie_a.spring_brake.outgrouped) as u8 as f32);
        self.a_lm_spring_brake_c
            .set_brightness(voltage * (light_test || bogie_c.spring_brake.outgrouped) as u8 as f32);
        self.a_lm_spring_brake_b
            .set_brightness(voltage * (light_test || bogie_b.spring_brake.outgrouped) as u8 as f32);

        //===============================================================
        // MMS communication
        //===============================================================

        mms_fault_sender.send(
            DiagnosticFaultKind::FederspeicherNichtGeloest,
            (bogie_a.spring_brake.state
                || bogie_b.spring_brake.state
                || bogie_c.spring_brake.state)
                && cab_a_runmode,
            None,
        );

        mms_fault_sender.send(
            DiagnosticFaultKind::FederspeicherAausgruppiert,
            bogie_a.spring_brake.outgrouped,
            Some(KeyEventCab::ACab),
        );
        mms_fault_sender.send(
            DiagnosticFaultKind::FederspeicherBausgruppiert,
            bogie_b.spring_brake.outgrouped,
            Some(KeyEventCab::ACab),
        );
        mms_fault_sender.send(
            DiagnosticFaultKind::FederspeicherCausgruppiert,
            bogie_c.spring_brake.outgrouped,
            Some(KeyEventCab::ACab),
        );

        mms_fault_sender.send(
            DiagnosticFaultKind::FederspeicherAgestoert,
            bogie_a.spring_brake.broken,
            Some(KeyEventCab::ACab),
        );
        mms_fault_sender.send(
            DiagnosticFaultKind::FederspeicherBgestoert,
            bogie_b.spring_brake.broken,
            Some(KeyEventCab::ACab),
        );
        mms_fault_sender.send(
            DiagnosticFaultKind::FederspeicherCgestoert,
            bogie_c.spring_brake.broken,
            Some(KeyEventCab::ACab),
        );

        fsp_car_offline || fsp_override
    }

    pub fn on_message(&mut self, msg: Message) {
        self.spring_brake_coupling.on_message(msg.clone());
    }
}

impl Default for SpringBrake {
    fn default() -> Self {
        Self::new()
    }
}

//====================================================================

pub struct BogieGt6n {
    spring_brake: SpringBrakeGt6n,

    powered_axis: ApiRailAxis,
    non_powered_axis: ApiRailAxis,

    snd_brake_flicker: Sound,
    snd_squeak_high: Sound,
    snd_squeak_low: Sound,
    snd_squeak_acc: Sound,
    snd_traction: Sound,

    snd_switch_rumbling: Sound,

    snd_curve_radius: Sound,

    snd_speed_at_non_powered_axis: Sound,
    snd_speed_at_powered_axis: Sound,

    pub p_target: f32,
    traction_force: f32,

    init_brake: f32,
}

impl BogieGt6n {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        bogie_index: usize,
        powered_axis_index: usize,
        non_powered_axis_index: usize,
        snd_brake_flicker_name: &str,
        snd_squeak_high_name: &str,
        snd_squeak_low_name: &str,
        snd_squeak_acc_name: &str,
        snd_traction_name: &str,
    ) -> Self {
        Self {
            spring_brake: SpringBrakeGt6n::new(powered_axis_index, bogie_index),

            powered_axis: ApiRailAxis::new(powered_axis_index, bogie_index),
            non_powered_axis: ApiRailAxis::new(non_powered_axis_index, bogie_index),

            snd_brake_flicker: Sound::new(None, Some(snd_brake_flicker_name), None),
            snd_squeak_high: Sound::new(None, Some(snd_squeak_high_name), None),
            snd_squeak_low: Sound::new(None, Some(snd_squeak_low_name), None),
            snd_squeak_acc: Sound::new(None, Some(snd_squeak_acc_name), None),
            snd_traction: Sound::new(None, Some(snd_traction_name), None),

            snd_switch_rumbling: Sound::new(
                None,
                Some(&format!("Snd_Rumpeln_Weiche_{}", bogie_index)),
                Some(&format!("Snd_Rumpeln_Pitch_{}", bogie_index)),
            ),

            snd_curve_radius: Sound::new(None, Some(&format!("invradius_{}", bogie_index)), None),

            snd_speed_at_non_powered_axis: Sound::new(
                None,
                Some(&format!(
                    "v_Axle_mps_{}_{}_abs",
                    bogie_index, non_powered_axis_index
                )),
                None,
            ),
            snd_speed_at_powered_axis: Sound::new(
                None,
                Some(&format!(
                    "v_Axle_mps_{}_{}_abs",
                    bogie_index, powered_axis_index
                )),
                None,
            ),

            p_target: 1.0,
            traction_force: 0.0,

            init_brake: 2.0,
        }
    }

    pub fn tick(
        &mut self,
        traction_target: f32,
        fast_brake: bool,
        anti_slip_activ: bool,
        loss_of_power: f32,
        spring_brake_target: bool,
        com: &mut Com,
    ) {
        // Read local signals
        let traction_voltage_norm = com.lv.get_or(WslTractionVoltageNorm, 0.0);
        let direction_of_driving = com
            .lv
            .get_or(WslDirectionOfDriving, DirectionOfDriving::default());
        let em_brakes = com.lv.get_or(WslEmergencyBrakes, false);
        let v_kmh = com.lv.get_or(WslSpeedometerKmh, 0.0);
        let v_ms_abs = (v_kmh / 3.6).abs();
        let v_ms = v_kmh / 3.6;

        let speed_in_direction_ms = v_ms * f32::from(direction_of_driving);

        let force_at_wheel = TRACTIONFORCE_N_ZERO - v_ms_abs * loss_of_power;

        // Read fuses

        // Input from key events

        // Input - Signale

        // Main logic

        // convert traction_targetgeber to target
        let target = if em_brakes || traction_target < 0.0 {
            if em_brakes || fast_brake {
                -1.0
            } else {
                traction_target * 1.1111
            }
        } else {
            traction_target
        };

        // Elektrische Fahrbremssteuerung
        let e_target = if target >= 0.0 {
            self.snd_brake_flicker.update_volume(0.0);
            target * f32::from(direction_of_driving) * force_at_wheel
        } else {
            let mut tmp_e_target = target.max(-1.0) * (v_ms_abs / E_BRAKE_LIMIT).min(1.0);
            if v_ms < 0.0 {
                tmp_e_target = -tmp_e_target;
            } else if anti_slip_activ {
                tmp_e_target /= 3.0;
            }
            self.snd_brake_flicker.update_volume(tmp_e_target);
            tmp_e_target * MAX_BRAKEFORCE_N
        };

        self.traction_force = if traction_voltage_norm > 0.8 {
            exponential_approach(self.traction_force, 10.0, e_target)
        } else {
            0.0
        };

        // Ansteuerung spring_brakebremse
        self.p_target = if target >= 0.0 {
            if speed_in_direction_ms < 0.0
                || (speed_in_direction_ms == 0.0
                    && ((target == 0.0) || (self.traction_force.abs() < (0.8 * e_target.abs()))))
            {
                1.0
            } else {
                0.0
            }
        } else if target > -1.1 {
            1.0 - (v_ms_abs / E_BRAKE_LIMIT).min(1.0)
        } else {
            1.0
        };

        self.p_target = if self.init_brake > 0.0 {
            self.init_brake -= delta();
            1.0
        } else {
            self.p_target
        };

        self.spring_brake.tick(spring_brake_target, self.p_target);

        // Gleitschutz einbringen
        if anti_slip_activ {
            self.p_target /= 3.0;
        }

        let pos_traction_force = self.traction_force.max(0.0).abs();
        let neg_traction_force = self.traction_force.min(0.0).abs();

        let traction = (pos_traction_force / TRACTIONFORCE_N_ZERO)
            .max(neg_traction_force)
            .abs()
            > (1.0 / TRACTIONFORCE_N_ZERO);

        let squeak_high = if traction && traction_target > 0.0 {
            1.0
        } else {
            (self.non_powered_axis.speed_mps() / 3.6).min(1.0)
        };

        let squeak_low = (traction && self.powered_axis.speed_mps() > 4.1667)
            || self.non_powered_axis.speed_mps() > 5.5555;

        // Kurvenquietschen
        let railquality_non_powered_axis = self.non_powered_axis.railquality();
        let railquality_powered_axis = self.powered_axis.railquality();

        let railquality_target = if railquality_non_powered_axis == RailQuality::FroggySmooth
            || railquality_non_powered_axis == RailQuality::FroggyRough
            || railquality_non_powered_axis == RailQuality::FlatGroove
            || railquality_powered_axis == RailQuality::FroggySmooth
            || railquality_powered_axis == RailQuality::FroggyRough
            || railquality_powered_axis == RailQuality::FlatGroove
        {
            SoundTarget::Start
        } else {
            SoundTarget::Stop
        };

        // Assign output
        self.powered_axis
            .set_tractionforce(self.traction_force * traction_voltage_norm);

        self.snd_traction.update_volume(self.traction_force);
        self.snd_squeak_acc
            .update_volume((pos_traction_force / TRACTIONFORCE_N_ZERO).abs());
        self.snd_squeak_high.update_volume(squeak_high);
        self.snd_squeak_low.update_volume(squeak_low as u8 as f32);

        self.snd_switch_rumbling.update_target(railquality_target);

        self.snd_switch_rumbling
            .update_pitch(0.9 + self.non_powered_axis.speed_mps() / 18.0);

        self.snd_curve_radius.update_volume(
            (self.non_powered_axis.invradius() + self.powered_axis.invradius()).abs() / 2.0,
        );

        self.snd_speed_at_non_powered_axis
            .update_volume(self.non_powered_axis.speed_mps().abs());
        self.snd_speed_at_powered_axis
            .update_volume(self.powered_axis.speed_mps().abs());
    }
}

//====================================================================

struct SpringBrakeGt6n {
    api_axis: ApiRailAxis,

    delay: Delay<bool>,

    state: bool,
    outgrouped: bool,
    broken: bool,
}

impl SpringBrakeGt6n {
    fn new(axis_id: usize, bogie_index: usize) -> Self {
        Self {
            api_axis: ApiRailAxis::new(axis_id, bogie_index),
            delay: Delay::new(0.3, false),
            state: false,
            outgrouped: false,
            broken: false,
        }
    }

    fn tick(&mut self, target: bool, p_brake: f32) {
        self.delay.tick(target);

        self.state = !self.outgrouped && self.delay.output;

        self.api_axis
            .set_brakeforce((self.state as u8 as f32).max(p_brake) * MAX_BRAKEFORCE_N);
    }
}
