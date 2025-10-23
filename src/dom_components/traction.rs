use std::collections::HashMap;

use lotus_script::{
    prelude::{Message, MessageTarget},
    time::delta,
};
use pandemist_vehicle_elements::{
    api::{
        axis::ApiRailAxis,
        key_event::KeyEventCab,
        mock_enums::VehicleInitState,
        simulation_settings::{init_car_is_reversed, init_pos_in_train, init_ready_state},
        sound::{Sound, SoundWithVol},
    },
    components::traction::{
        continous_throttle_lever::ContinuousThrottleLever, speedometer::Speedometer,
    },
    elements::{
        std::piecewise_linear_function::PiecewiseLinearFunction,
        tech::{
            key_switch::{KeyDepot, KeySwitch},
            switches::{StepSwitch, SwitchEventAction},
        },
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
        gt6n_coupling_messages::{
            CouplerCarActiv, CouplerReverser, CouplerThrottle, CouplerThrottleRear,
        },
        std_messages::{PowerSignalCabin, PowerSignalSender, VelocitySender},
    },
};

use crate::general::{
    local_values::{
        WslBatteryMainSwitch, WslBrakelight, WslCabState, WslDirectionOfDriving, WslDoorsClosed,
        WslElectricCoupled, WslEmergencyBrakes, WslFlapBlocked, WslSpeedometerKmh,
        WslTractionTarget, WslTractionVoltageNorm, WslTrainFormationSwitch, WslWorkshopKey,
    },
    setup::{const_veh_variant, FahrzeugVariante},
};

use super::sub_components::{
    anti_slide_anti_skid::AntiSlipAntiSlideProtectionUnit, bogie::Bogie, railbrakes::Railbrake,
    sanding::Sanding, sifa::Sifa,
};

pub struct Traction {
    mms_fault_sender: DiagnosticMessageSender,
    mainswitch_sender: PowerSignalSender,
    pub anti_slip_override: bool,
    speed_sender: VelocitySender,

    traction_control: TractionControl,

    km_h_last: f32,

    a_speedometer: Speedometer,
    a_speedometer_test_timer: f32,

    bogie: Bogie,
    railbrakes: Railbrake,
    sanding: Sanding,
    sifa: Sifa,
    anti_slip_anti_slide_unit: AntiSlipAntiSlideProtectionUnit,

    snd_stopjolt: Sound,

    snd_idle_i: Sound,
    snd_idle_vr: Sound,
    snd_ventilation: SoundWithVol,
}

impl Traction {
    pub fn new(driver_key: KeyDepot) -> Self {
        let mut s = Self {
            mms_fault_sender: DiagnosticMessageSender::default(),
            mainswitch_sender: PowerSignalSender::default(),
            anti_slip_override: false,

            speed_sender: VelocitySender::new(vec![MessageTarget::Broadcast {
                across_couplings: false,
                include_self: true,
            }]),

            traction_control: TractionControl::new(driver_key),

            km_h_last: 0.0,

            a_speedometer: Speedometer::builder("AV_A_Tachonadel")
                .force(20.0)
                .friction(4.0)
                .build(),
            a_speedometer_test_timer: 0.0,

            bogie: Bogie::new(),
            railbrakes: Railbrake::new(),
            sanding: Sanding::new(),
            sifa: Sifa::new(),
            anti_slip_anti_slide_unit: AntiSlipAntiSlideProtectionUnit::new(),

            snd_stopjolt: Sound::new_simple(Some("Snd_Halteruck")),

            snd_idle_i: Sound::new_simple(Some("Snd_Cabin_IdleI")),
            snd_idle_vr: Sound::new_simple(Some("Snd_Cabin_IdleVR")),
            snd_ventilation: SoundWithVol::new("Snd_Luefter_Vol", 1.0, 0.3),
        };

        s.mainswitch_sender.quickstart(
            s.traction_control.a_ignition_key.value(true) > 0
                && s.traction_control.a_reverser.value(true) > 0,
            0.into(),
        );

        s
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let battery =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let cab_a_runmode =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Star;
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let emergency_brake = com.lv.get_or(WslEmergencyBrakes, false);
        let workshop_key = com.lv.get_or(WslWorkshopKey(0), false);

        let fast_emergency_brake = emergency_brake
            || self.traction_control.traction_target < -0.95
            || self.sifa.forced_brake;

        // Tachogeber
        let km_h = ApiRailAxis::new(1, 0).speed_mps() * 3.6;
        com.lv.set(WslSpeedometerKmh, km_h);

        self.speed_sender
            .tick(km_h.abs() / 3.6 * f32::from(self.traction_control.direction_of_driving));

        // Read fuses

        // Input from key events
        self.traction_control
            .tick(&mut self.mainswitch_sender, &mut self.sifa, com);

        // Main logic

        // vMax Überwachung
        let v_max_mps = if workshop_key {
            70.0 / 3.6
        } else if self.traction_control.fallback_run_mode {
            30.0 / 3.6
        } else if self.traction_control.direction_of_driving.backward {
            15.0 / 3.6
        } else {
            60.0 / 3.6
        };

        // V Max Bremsung festlegen
        if cab_a_runmode {
            self.traction_control.v_max_warning = km_h > (v_max_mps * 3.6) + 1.0;
            if km_h > (v_max_mps * 3.6) + 5.0 {
                self.traction_control.v_max_brake = true;
            }
        } else {
            self.traction_control.v_max_warning = false;
            self.traction_control.v_max_brake = false;
        }

        // Tachometer
        self.do_tachmoeter(km_h, com);

        // Fahrmotor ansteuerung
        self.bogie.tick(
            self.traction_control.traction_target,
            self.traction_control.traction_target < -0.95,
            fast_emergency_brake,
            &self.anti_slip_anti_slide_unit,
            v_max_mps,
            com,
        );

        // Schienenbremsen ansteuerung
        self.railbrakes
            .tick(fast_emergency_brake, &self.anti_slip_anti_slide_unit, com);

        // Sand ansteugerung
        let v_kl_10 = km_h.abs() <= 10.0;
        let sand_override = (fast_emergency_brake && (km_h.abs() > 0.1))
            || (self.anti_slip_anti_slide_unit.anti_slide_active && v_kl_10);

        self.sanding.tick(sand_override, com);

        // Sifa ansteuerung
        self.sifa.tick(
            cab_a_runmode && !self.traction_control.fallback_run_mode,
            self.traction_control.a_throttle_lever.snappoint == 4,
            com,
        );

        // Gleit- & Schleuderschutz
        self.anti_slip_anti_slide_unit.tick(
            self.traction_control.traction_target,
            fast_emergency_brake,
            self.anti_slip_override,
            com,
        );

        // Assign output

        // Halteruck
        if km_h < 0.001 && self.km_h_last > 0.001 {
            self.snd_stopjolt.start();
        }

        self.snd_idle_i
            .start_stop(self.traction_control.car_activ && battery);
        self.snd_idle_vr.start_stop(
            self.traction_control.direction_of_driving.is_one()
                && self.traction_control.system_restart_timer < 0.0
                && battery,
        );
        self.snd_ventilation
            .tick(self.traction_control.direction_of_driving.is_one() && battery);

        self.km_h_last = km_h;

        //===============================================================
        // MMS communication
        //===============================================================

        self.mms_fault_sender.send(
            DiagnosticFaultKind::VmaxUeberschreitung,
            self.traction_control.v_max_warning,
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::AutomatischeBremsung,
            self.traction_control.v_max_brake,
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Anfahrsperre,
            self.traction_control.zeroing_condition_allgemein
                && self.traction_control.a_throttle_lever.pos > 0.0,
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::AnfahrsperreTueren,
            self.traction_control.zeroing_condition_freigabe
                && self.traction_control.a_throttle_lever.pos > 0.0,
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::MehrereFahrerstaendeAufgeruestet,
            cab_a_activ
                && (self.traction_control.car_activ_coupling.get_front()
                    || self.traction_control.car_activ_coupling.get_rear()),
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::Fahrtrichtungsfehler,
            self.traction_control.direction_of_driving.is_both(),
            None,
        );
    }

    fn do_tachmoeter(&mut self, km_h: f32, com: &mut Com) {
        let battery_on =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) == ChangedState::JustOn;

        if battery_on {
            self.a_speedometer_test_timer = 6.0;
        }

        if self.a_speedometer_test_timer > 0.0 {
            self.a_speedometer_test_timer -= delta();
        }

        if self.a_speedometer_test_timer > 3.0 {
            self.a_speedometer.tick(80.0, 0.0);
        } else if self.a_speedometer_test_timer > 0.0 {
            self.a_speedometer.tick(0.0, 0.0);
        } else {
            self.a_speedometer.tick(km_h.abs() / 80.0, 0.0);
        }
    }

    pub fn on_message(&mut self, msg: Message) {
        self.railbrakes.on_message(msg.clone());
        self.sanding.on_message(msg.clone());
        self.bogie.on_message(msg.clone());
        self.traction_control.on_message(msg.clone());
    }
}

pub struct TractionControl {
    car_activ_coupling: UniversalCouplingLine<bool, CouplerCarActiv>,
    reverser_coupling: UniversalCouplingLine<DirectionOfDriving, CouplerReverser>,
    throttle_coupling: UniversalCouplingLine<f32, CouplerThrottle>,
    throttle_rear_coupling: UniversalCouplingLine<f32, CouplerThrottleRear>,

    traction_target: f32,
    direction_of_driving: DirectionOfDriving,
    car_activ: bool,

    fallback_run_mode: bool,
    system_restart_timer: f32,

    zeroing_condition_allgemein: bool,
    zeroing_condition_freigabe: bool,
    v_max_warning: bool,
    v_max_brake: bool,
    v_tempomat: Option<f32>,
    tempomat_target: PiecewiseLinearFunction,

    a_throttle_lever: ContinuousThrottleLever,
    a_reverser: StepSwitch,
    a_ignition_key: KeySwitch,

    a_key_fallback_run_mode: KeySwitch,
    a_sw_fallback_driving_switch: StepSwitch,

    b_ignition_key: KeySwitch,
    b_sw_driving_switch: StepSwitch,
}

impl TractionControl {
    pub fn new(driver_key: KeyDepot) -> Self {
        let init_a = if init_pos_in_train() == 0 && !init_car_is_reversed() {
            init_ready_state()
        } else {
            VehicleInitState::ColdAndDark
        };

        let init_rw = if init_pos_in_train() == 0 && !init_car_is_reversed() {
            match init_ready_state() {
                VehicleInitState::ColdAndDark => 0,
                VehicleInitState::Setuped => 1,
                VehicleInitState::ReadyToDrive => 2,
            }
        } else {
            0
        };

        let init_b = if init_pos_in_train() == 0 && init_car_is_reversed() {
            init_ready_state()
        } else {
            VehicleInitState::ColdAndDark
        };

        Self {
            car_activ_coupling: UniversalCouplingLine::new(CouplerCarActiv {}, (true, true)),
            reverser_coupling: UniversalCouplingLine::new(CouplerReverser {}, (true, true)),
            throttle_coupling: UniversalCouplingLine::new(CouplerThrottle {}, (true, true)),
            throttle_rear_coupling: UniversalCouplingLine::new(
                CouplerThrottleRear {},
                (true, true),
            ),

            traction_target: 0.0,
            direction_of_driving: DirectionOfDriving::default(),
            car_activ: false,

            fallback_run_mode: false,
            system_restart_timer: 0.0,

            zeroing_condition_allgemein: false,
            zeroing_condition_freigabe: false,
            v_max_warning: false,
            v_max_brake: false,
            v_tempomat: None,
            tempomat_target: PiecewiseLinearFunction::new(vec![
                (100.0, -1.0),
                (-70.0, -1.0),
                (-50.0, -1.0),
                (-25.0, -0.8),
                (-15.0, -0.6),
                (-10.0, -0.4),
                (-5.0, -0.2),
                (0.0, 0.0),
                (5.0, 0.2),
                (10.0, 0.4),
                (15.0, 0.6),
                (25.0, 0.8),
                (50.0, 1.0),
                (70.0, 1.0),
                (100.0, 1.0),
            ]),

            a_throttle_lever: ContinuousThrottleLever::builder(
                "AV_A_Sollwertgeber",
                KeyEventCab::ACab,
            )
            .snd_notch_end("Snd_A_Sollwertgeber_End")
            .snd_notch_neutral("Snd_A_Sollwertgeber_NotchNeutral")
            .snd_notch_other("Snd_A_Sollwertgeber_NotchOther")
            .add_snappoint_config(0, -0.95)
            .add_snappoint_config(1, -0.87)
            .add_snappoint_config(2, -0.13)
            .add_snappoint_config(3, -0.05)
            .add_snappoint_config(4, 0.05)
            .add_snappoint_config(5, 0.13)
            .add_snappoint_config(6, 0.97)
            .add_snappoint_config(7, 2.0)
            .build(),

            a_reverser: StepSwitch::builder("AV_A_Richtungswender", Some(KeyEventCab::ACab))
                .event("ReverserPlus", SwitchEventAction::Plus)
                .event("ReverserMinus", SwitchEventAction::Minus)
                .snd_default_plus("Snd_A_Reverser")
                .snd_default_minus("Snd_A_Reverser")
                .min(0)
                .max(3)
                .mapping(HashMap::from([(0, 0.0), (1, 29.0), (2, 58.0), (3, 135.0)]))
                .init(init_rw)
                .build(),
            a_ignition_key: KeySwitch::builder(
                driver_key.clone(),
                "AV_A_Key_Betrieb",
                "vis_A_Key_Betrieb",
                Some(KeyEventCab::ACab),
            )
            .event_toggle("InsertKey_Reverser")
            .event_plus("Key_Reverser_R")
            .event_minus("Key_Reverser_L")
            .snd_insert("Snd_A_Key_Insert")
            .snd_takeout("Snd_A_Key_Takeout")
            .snd_default("Snd_A_Key_Turn")
            .pullout_min()
            .init(
                i32::from(init_a) > 0,
                ((init_a > VehicleInitState::ColdAndDark) as u8).into(),
            )
            .build(),

            a_key_fallback_run_mode: KeySwitch::builder(
                driver_key.clone(),
                "AV_A_Key_Notbetrieb",
                "vis_A_Key_Notbetrieb",
                Some(KeyEventCab::ACab),
            )
            .event_toggle("Key_Notbetrieb_Toggle")
            .event_turn("Key_Notbetrieb_Turn")
            .snd_insert("Snd_A_Key_Insert")
            .snd_insert("Snd_A_Key_Takeout")
            .snd_default("Snd_A_Key_Turn")
            .pullout_min()
            .pullout_max()
            .build(),

            a_sw_fallback_driving_switch: StepSwitch::builder(
                "AV_A_Sw_Notfahrschater",
                Some(KeyEventCab::ACab),
            )
            .event("ThrottleLeaverPlus", SwitchEventAction::Plus)
            .event("ThrottleLeaverMinus", SwitchEventAction::Minus)
            .event("Throttle", SwitchEventAction::Plus)
            .event("Neutral", SwitchEventAction::Set(1))
            .event("Brake", SwitchEventAction::Set(0))
            .event("MaxBrake", SwitchEventAction::Set(-1))
            .snd_default_plus("Snd_A_Switch")
            .snd_default_minus("Snd_A_Switch")
            .min(-1)
            .max(2)
            .max_spring()
            .mapping(HashMap::from([(1, 1.0), (0, 0.0), (1, -0.6), (2, -1.0)]))
            .build(),

            b_ignition_key: KeySwitch::builder(
                driver_key.clone(),
                "AV_B_Key_Betrieb",
                "vis_B_Key_Betrieb",
                Some(KeyEventCab::BCab),
            )
            .event_toggle("InsertKey_Reverser")
            .event_plus("Key_Reverser_R")
            .event_minus("Key_Reverser_L")
            .snd_insert("Snd_B_Key_Insert")
            .snd_takeout("Snd_B_Key_Takeout")
            .snd_default("Snd_B_Key_Turn")
            .pullout_min()
            .init(
                i32::from(init_b) > 0,
                ((init_b > VehicleInitState::ColdAndDark) as u8).into(),
            )
            .build(),

            b_sw_driving_switch: StepSwitch::builder("AV_B_Fahrschalter", Some(KeyEventCab::BCab))
                .event("ThrottleLeaverPlus", SwitchEventAction::Plus)
                .event("ThrottleLeaverMinus", SwitchEventAction::Minus)
                .event("Throttle", SwitchEventAction::Plus)
                .event("Neutral", SwitchEventAction::Set(1))
                .event("Brake", SwitchEventAction::Set(0))
                .event("MaxBrake", SwitchEventAction::Set(-1))
                .snd_default_plus("Snd_B_RotBtnOn")
                .snd_default_minus("Snd_B_RotBtnOff")
                .min(-1)
                .max(2)
                .max_spring()
                .mapping(HashMap::from([(1, -1.0), (0, 0.0), (1, 0.6), (2, 1.0)]))
                .build(),
        }
    }

    pub fn tick(
        &mut self,
        mainswitch_sender: &mut PowerSignalSender,
        sifa: &mut Sifa,
        com: &mut Com,
    ) {
        // Read local signals
        let battery =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);

        // Input from key events

        //----------------------------------------------------------------------
        // a tick
        //----------------------------------------------------------------------

        // a notfahrt
        self.a_key_fallback_run_mode.tick();
        self.fallback_run_mode = self.a_key_fallback_run_mode.value(battery) > 0;

        // a key
        let a_key_allowed = self.a_reverser.value(true) <= 1;
        if a_key_allowed {
            self.a_ignition_key.tick();
        }

        // a richtungswender
        let a_reverser_allowed =
            self.a_ignition_key.value(true) > 0 && self.a_throttle_lever.snappoint == 4;
        let a_reverser_last = self.a_reverser.value(true);
        if a_reverser_allowed {
            self.a_reverser.tick();
        }
        if a_reverser_last != self.a_reverser.value(true)
            && self.a_reverser.value(true) == 1
            && self.fallback_run_mode
        {
            self.system_restart_timer = 20.0;
        }

        // a throttle_lever
        let a_throttle_lever_allowed = self.a_reverser.value(true) > 1;
        if a_throttle_lever_allowed && !self.fallback_run_mode {
            self.a_throttle_lever.tick();
        }
        if a_throttle_lever_allowed && self.fallback_run_mode {
            self.a_sw_fallback_driving_switch.tick();
        }

        //----------------------------------------------------------------------
        // b tick
        //----------------------------------------------------------------------

        // b key
        let b_key_allowed = self.b_sw_driving_switch.value(true) == 0;
        if b_key_allowed {
            self.b_ignition_key.tick();
        }

        // b hilfsschalter
        let b_fahrschalter_allowed = self.b_ignition_key.value(true) > 0;
        if b_fahrschalter_allowed {
            self.b_sw_driving_switch.tick();
        }
        com.lv
            .set(WslFlapBlocked(1), self.b_ignition_key.is_inserted());

        // Main logic

        //----------------------------------------------------------------------
        // Aufrüstungszustand bestimmen
        //----------------------------------------------------------------------

        let a_cab_state = match self.a_reverser.value(true) {
            2 | 3 => CabActivState::VR,
            1 => CabActivState::Star,
            _ => CabActivState::Off,
        };
        com.lv.set(WslCabState(0), a_cab_state);

        let b_cab_state = match self.b_ignition_key.value(true) {
            1 => CabActivState::VR,
            _ => CabActivState::Off,
        };
        com.lv.set(WslCabState(1), b_cab_state);

        // Hilfsfahrerstand aufgerüstet zählt nicht als Aufgerüstet
        self.car_activ_coupling.update_permit(
            true,
            train_formation_switch != TrainFormationSwitch::Leading,
        );

        self.car_activ_coupling
            .update_local(a_cab_state > CabActivState::Off);
        self.car_activ = self.car_activ_coupling.get_value();

        mainswitch_sender.send(
            self.car_activ,
            if a_cab_state > CabActivState::Off {
                PowerSignalCabin::ACab
            } else {
                PowerSignalCabin::NoCab
            },
        );

        // System Restart
        if self.system_restart_timer > 0.0 {
            self.system_restart_timer -= delta();
        }

        self.control_target(sifa, com);
    }

    fn control_target(&mut self, sifa: &mut Sifa, com: &mut Com) {
        // Read local signals
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let cab_a_runmode =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Star;
        let cab_b_activ =
            com.lv.get_or(WslCabState(1), CabActivState::default()) > CabActivState::Off;
        let e_coupler_rear = com.lv.get_or(WslElectricCoupled(1), false);
        let doors_closed = com.lv.get_or(WslDoorsClosed, false);
        let traction_voltage = com.lv.get_or(WslTractionVoltageNorm, 0.0) > 0.8;
        let emergency_brake = com.lv.get_or(WslEmergencyBrakes, false);
        let km_h = com.lv.get_or(WslSpeedometerKmh, 0.0);
        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);

        let startup_interlock = !(doors_closed && traction_voltage);

        //----------------------------------------------------------------------
        // Anfahrsperren etc. auf den Sollwert anwenden
        //----------------------------------------------------------------------

        // Einfluss der Anfahrsperre
        if startup_interlock || emergency_brake {
            self.zeroing_condition_allgemein = true;
        }

        // Zwangsbremsung Grünschleife
        if !doors_closed {
            self.zeroing_condition_freigabe = true;
        }

        // Nullstellungen zurücksetzen
        if self.a_throttle_lever.snappoint == 4 {
            self.zeroing_condition_allgemein = false;
            self.zeroing_condition_freigabe = false;

            // V max Bremsung zurücksetzen
            if km_h < 0.01 {
                self.v_max_brake = false;
            }
        }

        //----------------------------------------------------------------------
        // Fahrtrichtung bestimmen
        //----------------------------------------------------------------------

        let direction_of_driving_front = DirectionOfDriving::new(
            self.a_reverser.value(true) == 2,
            self.a_reverser.value(true) == 3,
        );

        let direction_of_driving_rear = if cab_b_activ && !e_coupler_rear {
            DirectionOfDriving::new(false, true)
        } else {
            DirectionOfDriving::new(false, false)
        };

        let direction_of_driving_local =
            direction_of_driving_front.merge(&direction_of_driving_rear);

        // Signal noch durch die Zugsteuerleitung schicken
        self.reverser_coupling.update_permit(
            true,
            train_formation_switch != TrainFormationSwitch::Leading,
        );

        self.reverser_coupling
            .update_local(direction_of_driving_local);
        self.direction_of_driving = self.reverser_coupling.get_value();

        //----------------------------------------------------------------------
        // Sollwert bestimmen
        //----------------------------------------------------------------------

        let traction_target_front = if cab_a_runmode {
            if self.fallback_run_mode {
                match self.a_sw_fallback_driving_switch.value(true) {
                    2 => 0.5,
                    1 => 0.0,
                    -1 => -1.0,
                    _ => -0.5,
                }
            } else {
                let mut traction_target = self.a_throttle_lever.pos;

                // Tempomat bei GTU Fahrzeugen
                if const_veh_variant() == FahrzeugVariante::Gt6u {
                    if self.a_throttle_lever.snappoint == 5
                        && self.a_throttle_lever.snappoint_last != 5
                    {
                        self.v_tempomat = Some(km_h);
                    }
                    if self.a_throttle_lever.snappoint != 5 {
                        self.v_tempomat = None;
                    }

                    if let Some(vorgabe) = self.v_tempomat {
                        traction_target = self.tempomat_target.get_value_or_default(km_h - vorgabe);
                    }
                }

                traction_target
            }
        } else {
            0.0
        };

        // Signal vom Hilfsfahrerstand nur dann einspeisen, wenn er: aufgeschlossen ist, der letzte im Zug ist & die Fahrrichtug korrekt gesetzt wurde
        let traction_target_rear =
            if cab_b_activ && !e_coupler_rear && self.direction_of_driving.backward {
                match self.b_sw_driving_switch.value(true) {
                    2 => 0.5,
                    1 => 0.0,
                    -1 => -1.0,
                    _ => -0.5,
                }
            } else {
                0.0
            };

        self.throttle_rear_coupling.update_permit(
            true,
            train_formation_switch != TrainFormationSwitch::Leading,
        );

        // Nur das aufgerüstete Fahrzeug führt die auswertung durch
        let mut traction_target_local = if cab_a_activ {
            traction_target_front + traction_target_rear + self.throttle_rear_coupling.get_rear()
        } else {
            0.0
        };

        // Anfahrsperre Anwenden
        if startup_interlock
            || emergency_brake
            || sifa.zeroing_constrain
            || self.zeroing_condition_allgemein
            || self.zeroing_condition_freigabe
        {
            traction_target_local = traction_target_local.min(0.0);
        }

        // Zwangsbremsung anwenden
        if self.v_max_brake || sifa.forced_brake {
            traction_target_local = traction_target_local.min(-0.95);
        }

        // Sollwertausgabe blockieren, während die SPS neugestartet wird
        if self.system_restart_timer > 0.0 {
            traction_target_local = 0.0;
        }

        self.throttle_coupling.update_permit(
            true,
            train_formation_switch != TrainFormationSwitch::Leading,
        );

        self.throttle_coupling.update_local(traction_target_local);
        self.traction_target = self.throttle_coupling.get_value();

        com.lv.set(WslTractionTarget, self.traction_target);
        com.lv.set(WslBrakelight(1), self.traction_target < 0.0);
        com.lv.set(WslDirectionOfDriving, self.direction_of_driving);
    }

    pub fn on_message(&mut self, msg: Message) {
        self.car_activ_coupling.on_message(msg.clone());
        self.reverser_coupling.on_message(msg.clone());
        self.throttle_coupling.on_message(msg.clone());
        self.throttle_rear_coupling.on_message(msg.clone());
    }
}
