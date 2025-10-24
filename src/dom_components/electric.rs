use lotus_extra::messages::std_helper::BatteryvoltageSender;
use lotus_script::prelude::Message;
use pandemist_vehicle_elements::{
    api::{
        key_event::{KeyEvent, KeyEventCab},
        light::Light,
        mock_enums::VehicleInitState,
        simulation_settings::{init_pos_in_train, init_ready_state},
    },
    components::electrics::{
        converter::Converter,
        low_voltage_level::LowVoltageLevel,
        mainswitch::{CircuitBreaker, MainSwitch},
        pantograph::ElectricPantograph,
    },
    elements::{
        std::{piecewise_linear_function::PiecewiseLinearFunction, scroller::Pointer},
        tech::{
            key_switch::{KeyDepot, KeySwitch},
            switches::{StepSwitch, SwitchEventAction},
        },
    },
    management::{
        communicator::Com,
        enums::{
            general_enums::{CabActivState, TrainFormationSwitch},
            state_enums::{ChangedState, SwitchingState},
            target_enums::SwitchingTarget,
        },
    },
    messages::{
        diagnostic_messages::{
            DiagnosticFaultKind, DiagnosticMessageSender, DiagnosticPantoStateSender,
        },
        {coupling_handler::UniversalCouplingLine, gt6n_coupling_messages::CouplerPowerlinePower},
    },
};

use crate::general::{
    local_values::{
        WslBatteryMainSwitch, WslCabIndicatorBrightness, WslCabState, WslConverterVoltageNorm,
        WslElectricCoupled, WslLighttest, WslLowVoltageNorm, WslPermanentVoltageNorm,
        WslTractionVoltageNorm, WslTrainFormationSwitch,
    },
    setup::{const_veh_variant, FahrzeugVariante},
};

enum UsedMainswitch {
    Mainswitch {
        sw_mainswitch: Box<StepSwitch>,
        sw_mainswitch_override: KeyEvent,
        mainswitch_1: Box<MainSwitch>,
        mainswitch_2: Box<MainSwitch>,
        coupl_voltage_sender: UniversalCouplingLine<f32, CouplerPowerlinePower>,
        coupl_voltage_receiver: UniversalCouplingLine<f32, CouplerPowerlinePower>,
    },
    CircuitBreaker(CircuitBreaker),
}

pub struct Electric {
    mms_fault_sender: DiagnosticMessageSender,
    mms_panto_sender: DiagnosticPantoStateSender,

    battery_voltage_sender: BatteryvoltageSender,

    a_sw_pantograph: StepSwitch,

    a_sw_panto_override: KeyEvent,

    a_key_battery: KeySwitch,

    a_lm_no_mainswitch: Light,

    battery: LowVoltageLevel,
    converter: Converter,
    no_mainswitch: bool,
    mainswitch: UsedMainswitch,
    pantograph: ElectricPantograph,
    voltmeter: Pointer,
}

impl Electric {
    pub fn new(driver_key: KeyDepot) -> Self {
        let panto_init = init_ready_state() > VehicleInitState::ColdAndDark
            && (init_pos_in_train() == 0 || const_veh_variant() == FahrzeugVariante::Gt6u);

        let mainswitch = if const_veh_variant() != FahrzeugVariante::Gt6u {
            UsedMainswitch::Mainswitch {
                sw_mainswitch: Box::new(
                    StepSwitch::builder("AV_A_Sw_Hauptschalter", Some(KeyEventCab::ACab))
                        .event("HighVoltageMainSwitchOn", SwitchEventAction::Plus)
                        .event("HighVoltageMainSwitchOff", SwitchEventAction::Minus)
                        .snd_default_plus("Snd_A_RotBtnOn")
                        .snd_default_minus("Snd_A_RotBtnOff")
                        .min(-1)
                        .max(1)
                        .min_spring()
                        .max_spring()
                        .build(),
                ),

                sw_mainswitch_override: KeyEvent::new(
                    Some("HighVoltageMainSwitchToggle"),
                    Some(KeyEventCab::ACab),
                ),

                mainswitch_1: Box::new(
                    MainSwitch::builder(None)
                        .snd_turn_on("Snd_Hauptschalter_On")
                        .snd_turn_off("Snd_Hauptschalter_Off")
                        .init(panto_init)
                        .build(),
                ),
                mainswitch_2: Box::new(
                    MainSwitch::builder(None)
                        .snd_turn_on("Snd_Hauptschalter_On")
                        .snd_turn_off("Snd_Hauptschalter_Off")
                        .init(panto_init)
                        .build(),
                ),
                coupl_voltage_sender: UniversalCouplingLine::new(
                    CouplerPowerlinePower,
                    (false, true),
                ),
                coupl_voltage_receiver: UniversalCouplingLine::new(
                    CouplerPowerlinePower,
                    (true, false),
                ),
            }
        } else {
            UsedMainswitch::CircuitBreaker(CircuitBreaker::new())
        };

        //----------------------

        let pantograph = ElectricPantograph::builder(
            "AV_Pantograph_0",
            0,
            PiecewiseLinearFunction::new(vec![
                (3.4, 0.0),
                (4.0, 0.264),
                (4.5, 0.489),
                (5.0, 0.725),
                (5.5, 0.982),
            ]),
        )
        .snd_up("Snd_Panto_Up")
        .snd_down("Snd_Panto_Dn")
        .move_up_speed(1.0 / 4.56)
        .move_down_speed(1.0 / 5.60)
        .init(panto_init)
        .build();

        let mut mms_panto_sender = DiagnosticPantoStateSender::default();

        mms_panto_sender.send(pantograph.state);

        //----------------------

        let battery = LowVoltageLevel::builder(24.0)
            .voltage_max_v(26.0)
            .voltage_min_v(17.0)
            .voltage_loss_vs(0.005)
            .voltage_load_vs(24.0)
            .init(init_ready_state() > VehicleInitState::ColdAndDark)
            .build();

        let mut battery_voltage_sender = BatteryvoltageSender::default();

        battery_voltage_sender.send(battery.battery_mainswitch, battery.low_voltage_norm);

        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),
            mms_panto_sender,

            battery_voltage_sender,

            no_mainswitch: false,
            a_sw_pantograph: StepSwitch::builder("AV_A_Sw_Pantograph", Some(KeyEventCab::ACab))
                .event("PantographUp", SwitchEventAction::Plus)
                .event("PantographDn", SwitchEventAction::Minus)
                .snd_default_plus("Snd_A_RotBtnOn")
                .snd_default_minus("Snd_A_RotBtnOff")
                .min(-1)
                .max(1)
                .min_spring()
                .max_spring()
                .build(),

            a_sw_panto_override: KeyEvent::new(Some("PantographToggle"), Some(KeyEventCab::ACab)),

            a_key_battery: KeySwitch::builder(
                driver_key.clone(),
                "AV_A_Key_Batterie",
                "vis_A_Key_Batterie",
                Some(KeyEventCab::ACab),
            )
            .snd_insert("Snd_A_Key_Insert")
            .snd_takeout("Snd_A_Key_Takeout")
            .snd_default("Snd_A_Key_Turn")
            .event_toggle("Key_Batterie_Insert")
            .event_plus("Key_Batterie_Plus")
            .event_minus("Key_Batterie_Minus")
            .min(-1)
            .max(1)
            .add_pullout_state(0)
            .min_spring()
            .max_spring()
            .build(),

            a_lm_no_mainswitch: Light::new(Some("LM_A_Hauptschalter")),

            battery,

            converter: Converter::new(None, 0.75, 1.0, 4.0),
            mainswitch,
            pantograph,

            voltmeter: Pointer::new(25.0, 7.0, "AV_A_Voltmeternadel"),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let cab_indicator_light_level = com.lv.get_or(WslCabIndicatorBrightness(0), 1.0);
        let el_coupler_front = com.lv.get_or(WslElectricCoupled(0), false);
        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);

        // Read fuses
        let fuse_pantograph_motor = com.fuse.is_on("Stromabnehmerantrieb");
        let fuse_mainswitch1 = com.fuse.is_on("Hauptschalterantrieb1");
        let fuse_mainswitch2 = com.fuse.is_on("Hauptschalterantrieb2");
        let _fuse_mainswitch_tw1 = com.fuse.is_on("HauptschalterTW1");
        let _fuse_mainswitch_tw2 = com.fuse.is_on("HauptschalterTW2");
        let fuse_bnu = com.fuse.is_on("BordnetzumrichterDBU2");

        // Input from key events
        if self.a_sw_panto_override.is_just_pressed() {
            if self.pantograph.state == SwitchingState::On {
                self.a_sw_pantograph.key_minus.injection = true;
            } else {
                self.a_sw_pantograph.key_plus.injection = true;
            }
        }
        if self.a_sw_panto_override.is_released() {
            self.a_sw_pantograph.key_plus.injection = false;
            self.a_sw_pantograph.key_minus.injection = false;
        }

        self.a_sw_pantograph.tick();
        let mut panto_target = SwitchingTarget::new(self.a_sw_pantograph.value(cab_a_activ), 0.09);

        // Lower the current collector in the 2 carriage as soon as the control switch is set to 0
        if const_veh_variant() != FahrzeugVariante::Gt6u
            && el_coupler_front
            && !cab_a_activ
            && (self.pantograph.state > SwitchingState::Off
                || matches!(panto_target, SwitchingTarget::TurnOn(_)))
            && train_formation_switch > TrainFormationSwitch::Following
        {
            panto_target = SwitchingTarget::TurnOn(0.09);
        }

        self.a_key_battery.tick();
        let battery_target = SwitchingTarget::new(self.a_key_battery.value(true), 0.5);

        // Input - Signale

        // Main logic

        // Pantograph
        let panto_target = panto_target.and(fuse_pantograph_motor);

        self.pantograph.motor_target = panto_target;
        self.pantograph
            .tick(fuse_pantograph_motor, self.battery.battery_mainswitch);
        let panto_voltage = self.pantograph.voltage_norm;

        self.mms_panto_sender.send(self.pantograph.state);

        // Mainswitch/ Circuit breaker
        let traction_voltage;
        (self.no_mainswitch, traction_voltage) = match &mut self.mainswitch {
            UsedMainswitch::Mainswitch {
                sw_mainswitch,
                sw_mainswitch_override,
                mainswitch_1,
                mainswitch_2,
                coupl_voltage_sender,
                coupl_voltage_receiver,
            } => {
                if sw_mainswitch_override.is_just_pressed() {
                    if !self.no_mainswitch {
                        sw_mainswitch.key_minus.injection = true;
                    } else {
                        sw_mainswitch.key_plus.injection = true;
                    }
                }
                if sw_mainswitch_override.is_released() {
                    sw_mainswitch.key_plus.injection = false;
                    sw_mainswitch.key_minus.injection = false;
                }

                sw_mainswitch.tick();
                mainswitch_1.target = SwitchingTarget::new(sw_mainswitch.value(cab_a_activ), 0.13)
                    .and(fuse_mainswitch1);
                mainswitch_2.target = SwitchingTarget::new(sw_mainswitch.value(cab_a_activ), 0.13)
                    .and(fuse_mainswitch2);

                mainswitch_1.tick(panto_voltage);
                mainswitch_2.tick(panto_voltage);

                let spannung_output = if mainswitch_1.state {
                    mainswitch_1.output
                } else {
                    coupl_voltage_receiver.get_front()
                };

                com.lv.set(WslTractionVoltageNorm, spannung_output);
                // Transfer voltage to the rear carriage
                coupl_voltage_sender.update_local(mainswitch_2.output);
                (!mainswitch_1.state, spannung_output)
            }
            UsedMainswitch::CircuitBreaker(lss) => {
                lss.tick(panto_voltage);
                com.lv.set(WslTractionVoltageNorm, lss.output);
                (!lss.state, lss.output)
            }
        };

        // Converter
        self.converter.tick(traction_voltage, fuse_bnu);
        let converter_voltage = self.converter.ouput_voltage_norm;
        com.lv.set(WslConverterVoltageNorm, converter_voltage);

        // Battery
        self.battery.tick(converter_voltage, battery_target);
        com.lv.set(
            WslBatteryMainSwitch,
            ChangedState::to_changed(
                self.battery.battery_mainswitch,
                self.battery.battery_mainswitch_last,
            ),
        );

        self.battery_voltage_sender.send(
            self.battery.battery_mainswitch,
            self.battery.low_voltage_norm,
        );

        com.lv.set(WslLowVoltageNorm, self.battery.low_voltage_norm);
        com.lv
            .set(WslPermanentVoltageNorm, self.battery.permanent_voltage_norm);

        // Assign output
        self.voltmeter
            .tick(self.battery.low_voltage_norm / 40.0 * 24.0);

        self.a_lm_no_mainswitch.set_brightness(
            self.battery.low_voltage_norm
                * cab_indicator_light_level
                * ((cab_a_activ && self.no_mainswitch) || light_test) as u8 as f32,
        );

        //===============================================================
        // MMS communication
        //===============================================================

        self.mms_fault_sender.send(
            DiagnosticFaultKind::HauptschalterA,
            !(com.fuse.is_on("Hauptschalterantrieb1") && com.fuse.is_on("Hauptschalterantrieb2")),
            Some(KeyEventCab::ACab),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::HauptschalterAus,
            self.no_mainswitch && cab_a_activ,
            Some(KeyEventCab::ACab),
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::KeineFahrspannung,
            traction_voltage < 0.5,
            None,
        );
    }

    pub fn on_message(&mut self, msg: Message) {
        if let UsedMainswitch::Mainswitch {
            coupl_voltage_sender,
            coupl_voltage_receiver,
            ..
        } = &mut self.mainswitch
        {
            coupl_voltage_sender.on_message(msg.clone());
            coupl_voltage_receiver.on_message(msg.clone());
        }
    }
}
