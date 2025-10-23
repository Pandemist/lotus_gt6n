use lotus_script::{
    prelude::{Message, MessageTarget},
    time::delta,
};
use pandemist_vehicle_elements::{
    api::{
        key_event::{KeyEvent, KeyEventCab},
        light::Light,
        sound::Sound,
    },
    components::doors::aeg_electric_door::AegElectricDoor,
    elements::tech::{
        buttons::PushButton,
        seals::SealedSwitch,
        switches::{StepSwitch, Switch, SwitchEventAction},
    },
    management::{
        communicator::Com,
        enums::{
            door_enums::{DoorState, DoorTarget},
            general_enums::{CabActivState, TrainFormationSwitch},
            state_enums::ChangedState,
        },
    },
    messages::{
        coupling_handler::UniversalCouplingLine,
        diagnostic_messages::{
            DiagnosticDoorStateSender, DiagnosticFaultKind, DiagnosticMessageSender,
        },
        gt6n_coupling_messages::{CouplerDoorControl, CouplerDoorsClosed},
        std_messages::{DoorControlTarget, DoorSide, DoorStateSender},
    },
};

use crate::general::{
    local_values::{
        WslBatteryMainSwitch, WslCabIndicatorBrightness, WslCabOffButStillThere, WslCabState,
        WslDoorsClosed, WslEnergencyLight, WslInteriorLight, WslLighttest, WslLowVoltageNorm,
        WslPermanentVoltageNorm, WslSpeedometerKmh, WslTrainFormationSwitch,
        WslWheelchairHelperActive,
    },
    setup::{const_veh_variant, FahrzeugVariante},
};

use super::sub_components::{
    stop_request::StopRequest, warning_system::Warnanlage, whelechairhelper::WheelchairHelper,
};

pub enum UsedDoorControl {
    OldControls {
        sw_door: Box<StepSwitch>,
        release_override: KeyEvent,
        door1_override: KeyEvent,
    },
    NewControls {
        force_open_timer: f32,
        btn_gtu_door_release: Box<PushButton>,
        btn_gtu_door_close: Box<PushButton>,
        btn_gtu_door1: Box<PushButton>,
        release_override: KeyEvent,
        force_open_override: KeyEvent,
        force_close_override: KeyEvent,
    },
}

pub struct Doors {
    mms_fault_sender: DiagnosticMessageSender,
    mms_door_state_sender: DiagnosticDoorStateSender,
    door_state_sender: DoorStateSender,

    door_control_send_coupling: UniversalCouplingLine<DoorTarget, CouplerDoorControl>,
    door_control_receive_coupling: UniversalCouplingLine<DoorTarget, CouplerDoorControl>,
    doors_closed_send_coupling: UniversalCouplingLine<bool, CouplerDoorsClosed>,
    doors_closed_receive_coupling: UniversalCouplingLine<bool, CouplerDoorsClosed>,

    stop_request: StopRequest,
    warnanalage: Warnanlage,
    doorlight: DoorLight,

    // emergency unlocks
    emergency_door_unlock_1: Switch,
    emergency_door_unlock_2: Switch,
    emergency_door_unlock_3: Switch,
    emergency_door_unlock_4: Switch,

    // Doors
    door_1: AegElectricDoor,
    door_2: AegElectricDoor,
    door_3: AegElectricDoor,
    door_4: AegElectricDoor,

    // States
    door_1_flip_flop: bool,
    door_1_flip_flop_last: bool,
    door_4_flip_flop: bool,
    door_4_flip_flop_last: bool,

    door_1_target: DoorTarget,
    door_local_target: DoorTarget,
    door_target: DoorTarget,
    door_2_target: DoorTarget,
    door_3_target: DoorTarget,
    door_4_target: DoorTarget,
    force_close_requested: bool,
    doors_closed_a_last: bool,
    door_switch_blocked: bool,
    door_switch_too_early: bool,
    battery_off_doorlight_timer: f32,
    cab_shutoff_flag: bool,
    cab_a_activ_last: bool,
    door_contactor: bool,
    had_stop_request_1: bool,
    had_stop_request_2: bool,
    had_stop_request_3: bool,
    had_stop_request_4: bool,

    // Einstiegshilfe
    wheelchair_helper: WheelchairHelper,
    lift_request: bool,
    lift_request_granted: bool,
    wheelchair_helper_ready: bool,
    lift_ramp_in_use: bool,

    a_door_control: UsedDoorControl,

    a_sw_bypass_switch_doors_closed_seal: SealedSwitch,

    a_lm_doors_closed: Light,

    a_snd_pling: Sound,

    b_btn_tuer_4: PushButton,

    snd_door_contactor_on: Sound,
    snd_door_contactor_off: Sound,
}

impl Doors {
    pub fn new() -> Self {
        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),
            mms_door_state_sender: DiagnosticDoorStateSender::default(),
            door_state_sender: DoorStateSender::new(vec![MessageTarget::Broadcast {
                across_couplings: false,
                include_self: true,
            }]),

            door_control_send_coupling: UniversalCouplingLine::new(
                CouplerDoorControl,
                (false, true),
            ),
            door_control_receive_coupling: UniversalCouplingLine::new(
                CouplerDoorControl,
                (true, false),
            ),
            doors_closed_send_coupling: UniversalCouplingLine::new(
                CouplerDoorsClosed,
                (true, false),
            ),
            doors_closed_receive_coupling: UniversalCouplingLine::new(
                CouplerDoorsClosed,
                (false, true),
            ),

            stop_request: StopRequest::new(),
            warnanalage: Warnanlage::new(),
            doorlight: DoorLight::new(),

            // Notentriegelungen
            emergency_door_unlock_1: Switch::builder("AV_Notentriegelung_1", None)
                .event_toggle("Notentriegelung_1")
                .build(),
            emergency_door_unlock_2: Switch::builder("AV_Notentriegelung_2", None)
                .event_toggle("Notentriegelung_2")
                .build(),
            emergency_door_unlock_3: Switch::builder("AV_Notentriegelung_3", None)
                .event_toggle("Notentriegelung_3")
                .build(),
            emergency_door_unlock_4: Switch::builder("AV_Notentriegelung_4", None)
                .event_toggle("Notentriegelung_4")
                .build(),

            // Tür
            door_1: AegElectricDoor::builder(1, "AV_Door_1X", "AV_Door_1")
                .mouse_factor(0.1)
                .add_warning("L_Door_1_Warnlicht_Innen", "Snd_Door_1_Warning")
                .set_1st_series(
                    "Snd_Door_1_Open_Start",
                    "Snd_Door_1_Open_End",
                    "Snd_Door_1_Close_Start",
                    "Snd_Door_1_Close_Trans",
                    "Snd_Door_1_Close_End",
                )
                .build(),
            door_2: AegElectricDoor::builder(2, "AV_Door_2X", "AV_Door_2")
                .mouse_factor(0.1)
                .add_warning("L_Door_2_Warnlicht_Innen", "Snd_Door_2_Warning")
                .set_1st_series(
                    "Snd_Door_2_Open_Start",
                    "Snd_Door_2_Open_End",
                    "Snd_Door_2_Close_Start",
                    "Snd_Door_2_Close_Trans",
                    "Snd_Door_2_Close_End",
                )
                .build(),
            door_3: AegElectricDoor::builder(3, "AV_Door_3X", "AV_Door_3")
                .mouse_factor(0.1)
                .add_warning("L_Door_3_Warnlicht_Innen", "Snd_Door_3_Warning")
                .set_1st_series(
                    "Snd_Door_3_Open_Start",
                    "Snd_Door_3_Open_End",
                    "Snd_Door_3_Close_Start",
                    "Snd_Door_3_Close_Trans",
                    "Snd_Door_3_Close_End",
                )
                .build(),
            door_4: AegElectricDoor::builder(4, "AV_Door_4X", "AV_Door_4")
                .mouse_factor(0.1)
                .add_warning("L_Door_4_Warnlicht_Innen", "Snd_Door_4_Warning")
                .set_1st_series(
                    "Snd_Door_4_Open_Start",
                    "Snd_Door_4_Open_End",
                    "Snd_Door_4_Close_Start",
                    "Snd_Door_4_Close_Trans",
                    "Snd_Door_4_Close_End",
                )
                .build(),

            // Melde Zuständ
            door_1_flip_flop: false,
            door_1_flip_flop_last: false,
            door_4_flip_flop: false,
            door_4_flip_flop_last: false,

            door_local_target: DoorTarget::FastClose,
            door_target: DoorTarget::FastClose,
            door_1_target: DoorTarget::FastClose,
            door_2_target: DoorTarget::FastClose,
            door_3_target: DoorTarget::FastClose,
            door_4_target: DoorTarget::FastClose,
            force_close_requested: false,
            doors_closed_a_last: false,
            door_switch_blocked: false,
            door_switch_too_early: false,
            battery_off_doorlight_timer: 0.0,
            cab_shutoff_flag: false,
            cab_a_activ_last: false,
            door_contactor: false,
            had_stop_request_1: false,
            had_stop_request_2: false,
            had_stop_request_3: false,
            had_stop_request_4: false,

            wheelchair_helper: WheelchairHelper::new(),
            lift_request: false,
            lift_request_granted: false,
            wheelchair_helper_ready: false,
            lift_ramp_in_use: false,

            a_door_control: if const_veh_variant() != FahrzeugVariante::Gt6u {
                UsedDoorControl::OldControls {
                    sw_door: Box::new(
                        StepSwitch::builder("AV_A_Sw_Tuersteuerung", Some(KeyEventCab::ACab))
                            .event("DoorAllClose", SwitchEventAction::Set(0))
                            .event("DoorAllOpen", SwitchEventAction::Set(2))
                            .event("DoorPlus", SwitchEventAction::Plus)
                            .event("DoorMinus", SwitchEventAction::Minus)
                            .snd_default_plus("Snd_A_Switch")
                            .snd_default_minus("Snd_A_Switch")
                            .min(-1)
                            .max(2)
                            .min_spring()
                            .build(),
                    ),
                    release_override: KeyEvent::new(
                        Some("DoorReleaseToggle"),
                        Some(KeyEventCab::ACab),
                    ),
                    door1_override: KeyEvent::new(Some("Door1Toggle"), Some(KeyEventCab::ACab)),
                }
            } else {
                UsedDoorControl::NewControls {
                    force_open_timer: 0.0,
                    btn_gtu_door_release: Box::new(
                        PushButton::builder(
                            "AV_A_Btn_Tuerfreigabe",
                            "DoorReleaseOn",
                            Some(KeyEventCab::ACab),
                        )
                        .snd_press("Snd_A_BtnDn")
                        .snd_release("Snd_A_BtnUp")
                        .build(),
                    ),
                    btn_gtu_door_close: Box::new(
                        PushButton::builder(
                            "AV_A_Btn_Gruenschleife",
                            "DoorReleaseOff",
                            Some(KeyEventCab::ACab),
                        )
                        .snd_press("Snd_A_BtnDn")
                        .snd_release("Snd_A_BtnUp")
                        .build(),
                    ),
                    btn_gtu_door1: Box::new(
                        PushButton::builder(
                            "AV_A_Sw_Tuer1",
                            "Door1Toggle",
                            Some(KeyEventCab::ACab),
                        )
                        .snd_press("Snd_A_BtnDn")
                        .snd_release("Snd_A_BtnUp")
                        .build(),
                    ),
                    release_override: KeyEvent::new(
                        Some("DoorReleaseToggle"),
                        Some(KeyEventCab::ACab),
                    ),
                    force_open_override: KeyEvent::new(
                        Some("DoorAllOpen"),
                        Some(KeyEventCab::ACab),
                    ),
                    force_close_override: KeyEvent::new(
                        Some("DoorAllClose"),
                        Some(KeyEventCab::ACab),
                    ),
                }
            },

            a_sw_bypass_switch_doors_closed_seal: SealedSwitch::new(
                Some(KeyEventCab::ACab),
                "vis_A_Plombe_Hilfsschalter_Gruenschleife",
                "Plombe_Hilfsschalter_Gruenschleife",
                Switch::builder(
                    "AV_A_Sw_Hilfsschalter_Gruenschleife",
                    Some(KeyEventCab::ACab),
                )
                .event_toggle("Hilfsschalter_Gruenschleife")
                .snd_toggle("Snd_A_Switch")
                .build(),
            ),

            a_lm_doors_closed: Light::new(Some("LM_A_Gruenschleife")),

            a_snd_pling: Sound::new_simple(Some("Snd_A_Abfahrtspling")),

            b_btn_tuer_4: PushButton::builder(
                "AV_B_Btn_Tueren_4",
                "Door1Toggle",
                Some(KeyEventCab::BCab),
            )
            .snd_press("Snd_B_BtnDn")
            .snd_release("Snd_B_BtnUp")
            .build(),

            snd_door_contactor_on: Sound::new_simple(Some("Snd_Richtungsschuetze_On")),
            snd_door_contactor_off: Sound::new_simple(Some("Snd_Richtungsschuetze_Off")),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let battery =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let cab_a_runmode =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Star;
        let cab_b_activ =
            com.lv.get_or(WslCabState(1), CabActivState::default()) > CabActivState::Off;
        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let cab_indicator_light_level = com.lv.get_or(WslCabIndicatorBrightness(0), 1.0);
        let km_h = com.lv.get_or(WslSpeedometerKmh, 0.0);

        // Read fuses
        let fuse_door_1_power = com.fuse.is_on("VersorgungTuer1");

        // Input from key events

        match &mut self.a_door_control {
            UsedDoorControl::OldControls {
                sw_door: switch,
                release_override,
                door1_override,
            } => {
                // Wenn Num / gedrückt wird, den Switch drehen
                if door1_override.is_just_pressed() && switch.value(true) >= 0 {
                    switch.set(-1);
                }
                // Wenn Num / gedrückt wird, den Switch drehen
                if door1_override.is_just_released() && switch.value(true) < 0 {
                    switch.set(0);
                }
                // Wenn Num - gedrückt wird, den Switch drehen
                if release_override.is_just_pressed() {
                    if switch.value(true) != 0 {
                        switch.set(0);
                    } else {
                        switch.set(1);
                    }
                }
                switch.tick()
            }
            UsedDoorControl::NewControls {
                mut force_open_timer,
                btn_gtu_door_release,
                btn_gtu_door_close,
                btn_gtu_door1,
                release_override,
                force_open_override,
                force_close_override,
            } => {
                if force_open_override.is_just_pressed() {
                    force_open_timer = 1.5;
                }

                if force_open_timer > 0.0 {
                    force_open_timer -= delta();
                }

                if force_open_timer > 0.0
                    || self.door_target != DoorTarget::Open
                    || self.force_close_requested
                {
                    force_open_timer = -1.0;
                    btn_gtu_door_release.key_press.injection = false;
                    btn_gtu_door_close.key_press.injection = false;
                }

                btn_gtu_door_release.key_press.injection = force_open_timer < 1.4
                    && force_open_timer > 0.9
                    || force_open_timer < 0.5 && force_open_timer > 0.0;

                btn_gtu_door_close.key_press.injection = force_close_override.is_just_pressed();

                if release_override.is_just_pressed() {
                    if self.door_target > DoorTarget::Close {
                        btn_gtu_door_close.key_press.injection = true;
                    } else {
                        btn_gtu_door_release.key_press.injection = true;
                    }
                }
                // Wenn Num - losgelassen wird, beide
                if release_override.is_just_released() {
                    btn_gtu_door_release.key_press.injection = false;
                    btn_gtu_door_close.key_press.injection = false;
                }
                btn_gtu_door_release.tick();
                btn_gtu_door_close.tick();
                btn_gtu_door1.tick();
            }
        }

        self.a_sw_bypass_switch_doors_closed_seal.tick();

        self.b_btn_tuer_4.tick();

        // Anpassung des Targets wegen besonderem verhalten aus: https://www.lotus-simulator.de/forum/index.php?thread/1611-gt6-t%C3%BCrstatus-bei-anfahrt/&postID=14204#post14204
        if self.door_switch_too_early {
            self.had_stop_request_1 = self.had_stop_request_1 || self.stop_request.request_1;
            self.had_stop_request_2 = self.had_stop_request_2 || self.stop_request.request_2;
            self.had_stop_request_3 = self.had_stop_request_3 || self.stop_request.request_3;
            self.had_stop_request_4 = self.had_stop_request_4 || self.stop_request.request_4;
        } else {
            self.had_stop_request_1 = false;
            self.had_stop_request_2 = false;
            self.had_stop_request_3 = false;
            self.had_stop_request_4 = false;
        }

        let local_target_1 = if self.lift_request_granted || self.stop_request.lift_allowed {
            // Force open, wenn der Hublift angefordert wurde
            DoorTarget::Open
        } else if self.door_switch_too_early && !self.had_stop_request_1 {
            self.door_1_target.max(DoorTarget::Release)
        } else {
            self.door_1_target
        };
        let local_target_2 = if self.door_switch_too_early && !self.had_stop_request_2 {
            self.door_2_target.max(DoorTarget::Release)
        } else {
            self.door_2_target
        };
        let local_target_3 = if self.door_switch_too_early && !self.had_stop_request_3 {
            self.door_3_target.max(DoorTarget::Release)
        } else {
            self.door_3_target
        };
        let local_target_4 = if self.door_switch_too_early && !self.had_stop_request_4 {
            self.door_4_target.max(DoorTarget::Release)
        } else {
            self.door_4_target
        };

        // Zwangsaufhalten für Rolli und Kiwa Taster einließen lassen
        if self.door_1_target == DoorTarget::Release && self.stop_request.force_open_1 {
            self.door_1_target = DoorTarget::Open;
        }
        if self.door_2_target == DoorTarget::Release && self.stop_request.force_open_2 {
            self.door_2_target = DoorTarget::Open;
        }
        if self.door_3_target == DoorTarget::Release && self.stop_request.force_open_3 {
            self.door_3_target = DoorTarget::Open;
        }
        if self.door_4_target == DoorTarget::Release && self.stop_request.force_open_4 {
            self.door_4_target = DoorTarget::Open;
        }

        // Main logic
        self.lift_request = const_veh_variant() != FahrzeugVariante::Gt6u
            && self.wheelchair_helper.lift_requested();

        self.lift_request_granted = const_veh_variant() != FahrzeugVariante::Gt6u
            && ((self.lift_request && km_h.abs() < 0.6) || self.wheelchair_helper.in_use());

        self.wheelchair_helper_ready = if const_veh_variant() != FahrzeugVariante::Gt6u {
            self.door_1.state == DoorState::Open && self.lift_request
        } else {
            self.door_1.state == DoorState::Open
                && self.stop_request.lift_allowed
                && self.lift_request
                && self.lift_request_granted
        };

        com.lv.set(
            WslWheelchairHelperActive(0),
            self.lift_request_granted || self.stop_request.lift_allowed,
        );

        self.wheelchair_helper
            .tick(self.wheelchair_helper_ready, com);

        self.lift_ramp_in_use = self.wheelchair_helper.in_use();

        self.global_door_state(com);
        self.stop_request.tick(
            [
                self.door_1.state,
                self.door_2.state,
                self.door_3.state,
                self.door_4.state,
            ],
            self.door_target,
            self.lift_request_granted,
            com,
        );

        if self.battery_off_doorlight_timer > 0.0 {
            self.battery_off_doorlight_timer -= delta();
        }

        self.doorlight.tick(
            [
                self.door_1.state,
                self.door_2.state,
                self.door_3.state,
                self.door_4.state,
            ],
            self.battery_off_doorlight_timer,
            com,
        );

        // Abrüstflag updaten
        if self.cab_a_activ_last && !cab_a_activ {
            self.cab_shutoff_flag = true;
        }
        if self.door_1.state > DoorState::Closed {
            self.cab_shutoff_flag = false;
        }
        com.lv.set(WslCabOffButStillThere(0), self.cab_shutoff_flag);

        // Notentrieglungen
        self.emergency_door_unlock_1.tick();
        self.emergency_door_unlock_2.tick();
        self.emergency_door_unlock_3.tick();
        self.emergency_door_unlock_4.tick();

        // Türen
        self.door_1.tick(
            battery && fuse_door_1_power,
            local_target_1,
            false,
            self.emergency_door_unlock_1.value(true),
            self.stop_request.request_1,
        );

        self.door_2.tick(
            battery,
            local_target_2,
            false,
            self.emergency_door_unlock_2.value(true),
            self.stop_request.request_2,
        );

        self.door_3.tick(
            battery,
            local_target_3,
            false,
            self.emergency_door_unlock_3.value(true),
            self.stop_request.request_3,
        );

        self.door_4.tick(
            battery,
            local_target_4,
            false,
            self.emergency_door_unlock_4.value(true),
            self.stop_request.request_4,
        );

        // Grünschleife bestimmen
        let doors_closed = (self.door_1.state == DoorState::Closed
            && self.door_2.state == DoorState::Closed
            && self.door_3.state == DoorState::Closed
            && self.door_4.state == DoorState::Closed
            && self.door_target <= DoorTarget::Close)
            || self.a_sw_bypass_switch_doors_closed_seal.switch.value(true);

        //set_var(
        //    "AA_gruenschleife_send_allowed",
        //    format!("{:?}", self.doors_closed_send_coupling.is_allowed),
        //);
        //set_var(
        //    "AA_gruenschleife_send",
        //    format!("{:?}", self.doors_closed_send_coupling.local_value),
        //);
        //set_var(
        //    "AA_gruenschleife_send_last_send",
        //    format!("{:?}", self.doors_closed_send_coupling.last_send),
        //);

        //set_var(
        //    "AA_gruenschleife_rcv_allowed",
        //    format!("{:?}", self.doors_closed_receive_coupling.is_allowed),
        //);
        //set_var(
        //    "AA_gruenschleife_rcv",
        //    format!("{:?}", self.doors_closed_receive_coupling.get_rear()),
        //);
        //set_var(
        //    "AA_gruenschleife_rcv_last_rcved",
        //    format!("{:?}", self.doors_closed_receive_coupling.received),
        //);

        // Signal nach vorne auf die Zugsteuerleitung geben
        self.doors_closed_send_coupling.update_local(doors_closed);
        // Signal von hinten von der Zugsteuerleitung nehmen (nur bei 1+2 Betrieb)
        let doors_closed = if train_formation_switch != TrainFormationSwitch::Leading {
            doors_closed && self.doors_closed_receive_coupling.get_rear()
        } else {
            doors_closed
        };

        com.lv.set(WslDoorsClosed, doors_closed);

        if cab_a_runmode && doors_closed && !self.doors_closed_a_last {
            self.a_snd_pling.start();
            self.doors_closed_a_last = true;
        }
        if !cab_a_runmode || !doors_closed {
            self.doors_closed_a_last = false;
        }

        // Warnanlage
        let warnanlage_aktiv = self.warnanalage.tick(
            self.door_target,
            self.force_close_requested,
            self.wheelchair_helper.blink_relais(),
            com,
        );

        // Assign output
        self.door_1.warn_tick(battery, warnanlage_aktiv, voltage);
        self.door_2.warn_tick(battery, warnanlage_aktiv, voltage);
        self.door_3.warn_tick(battery, warnanlage_aktiv, voltage);
        self.door_4.warn_tick(battery, warnanlage_aktiv, voltage);

        self.a_lm_doors_closed.set_brightness(
            voltage
                * cab_indicator_light_level
                * (light_test || (doors_closed && cab_a_runmode)) as u8 as f32,
        );

        // Richtungsschuetz
        if doors_closed && (cab_a_runmode || cab_b_activ) && !self.door_contactor {
            self.door_contactor = true;
            self.snd_door_contactor_on.start();
        }

        if (km_h < 1.0 && self.door_target > DoorTarget::Close)
            && (!cab_a_runmode && !cab_b_activ)
            && self.door_contactor
        {
            self.door_contactor = false;
            self.snd_door_contactor_off.start();
        }

        //===============================================================
        // MMS communication
        //===============================================================

        let mms_door_state = if self.door_1.state == DoorState::Open
            || self.door_2.state == DoorState::Open
            || self.door_3.state == DoorState::Open
            || self.door_4.state == DoorState::Open
        {
            DoorTarget::Open
        } else if self.door_target >= DoorTarget::Release
            || (self.door_1.state != DoorState::Closed
                || self.door_2.state != DoorState::Closed
                || self.door_3.state != DoorState::Closed
                || self.door_4.state != DoorState::Closed)
        {
            DoorTarget::Release
        } else {
            DoorTarget::Close
        };

        self.mms_door_state_sender.send(mms_door_state);
        self.door_state_sender.send(match self.door_target {
            DoorTarget::FastClose => DoorControlTarget::Closed,
            DoorTarget::Close => DoorControlTarget::Closed,
            DoorTarget::Release => DoorControlTarget::Released(DoorSide::Right),
            DoorTarget::Open => DoorControlTarget::Open(DoorSide::Right),
        });

        self.mms_fault_sender.send(
            DiagnosticFaultKind::GruenschleifeUeberbrueckt,
            self.a_sw_bypass_switch_doors_closed_seal
                .switch
                .value(cab_a_activ),
            None,
        );

        self.mms_fault_sender.send(
            DiagnosticFaultKind::NotentriegelungR1,
            self.emergency_door_unlock_1.value(true),
            None,
        );
        self.mms_fault_sender.send(
            DiagnosticFaultKind::NotentriegelungR2,
            self.emergency_door_unlock_2.value(true),
            None,
        );
        self.mms_fault_sender.send(
            DiagnosticFaultKind::NotentriegelungR3,
            self.emergency_door_unlock_3.value(true),
            None,
        );
        self.mms_fault_sender.send(
            DiagnosticFaultKind::NotentriegelungR4,
            self.emergency_door_unlock_4.value(true),
            None,
        );
    }

    fn global_door_state(&mut self, com: &mut Com) {
        // Read local signals
        let battery =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;
        let cab_b_activ =
            com.lv.get_or(WslCabState(1), CabActivState::default()) > CabActivState::Off;
        let km_h = com.lv.get_or(WslSpeedometerKmh, 0.0);
        let train_formation_switch = com
            .lv
            .get_or(WslTrainFormationSwitch(0), TrainFormationSwitch::Leading);

        // Read fuses
        let fuse_door_1_power = com.fuse.is_on("VersorgungTuer1");
        let fuse_central_door_control = com.fuse.is_on("ZentraleTuersteuerung");

        // Vorwerte setzen

        // Zentrale Tüeransteuerung
        match &mut self.a_door_control {
            UsedDoorControl::OldControls {
                sw_door: step_switch,
                ..
            } => {
                self.door_local_target = match step_switch.value(true) {
                    2 => DoorTarget::Open,
                    1 => DoorTarget::Release,
                    _ => DoorTarget::Close,
                };

                if step_switch.just_changed_to(true, -1) {
                    self.door_1_flip_flop = !self.door_1_flip_flop;
                }

                self.door_switch_blocked = (self.door_switch_blocked || (km_h.abs() > 6.0))
                    && (step_switch.value(true) != 0);

                self.door_switch_too_early = (self.door_switch_too_early || (km_h.abs() > 0.6))
                    && (step_switch.value(true) != 0);
            }
            UsedDoorControl::NewControls {
                btn_gtu_door_release,
                btn_gtu_door_close,
                ..
            } => {
                if btn_gtu_door_release.is_just_pressed() {
                    if self.door_local_target == DoorTarget::Release {
                        self.door_local_target = DoorTarget::Open;
                    } else {
                        self.door_local_target = DoorTarget::Release;
                    }
                }

                if btn_gtu_door_close.is_just_pressed() {
                    self.door_local_target = DoorTarget::Close;
                }
            }
        }

        // Pre Target nachbearbeiten
        if self.door_switch_blocked {
            self.door_local_target = DoorTarget::Close;
        }

        if !cab_a_activ {
            self.door_local_target = self.door_local_target.max(DoorTarget::Close);
        }

        // Signal nach hinten auf die Steuerleitung geben (Master)
        self.door_control_send_coupling.update_permit(
            false,
            train_formation_switch != TrainFormationSwitch::Leading,
        );

        self.door_control_send_coupling
            .update_local(self.door_local_target);

        let door_target_last = self.door_target;

        // Signal von vorne aus der Steuerleitung nehmen
        self.door_target = self
            .door_local_target
            .merge(&self.door_control_receive_coupling.get_front());

        // Tür 4
        if cab_b_activ && self.b_btn_tuer_4.is_just_pressed() {
            self.door_4_flip_flop = !self.door_4_flip_flop;
        }

        if !cab_b_activ {
            self.door_4_flip_flop = false;
        }

        // Gedönz überschreiben

        // Tür 1, wenn die Batterie aus ist
        match &mut self.a_door_control {
            UsedDoorControl::OldControls {
                sw_door: step_switch,
                ..
            } => {
                if step_switch.just_changed_to(true, -1) && !battery {
                    if self.door_1_flip_flop {
                        if self.battery_off_doorlight_timer <= 0.0 {
                            self.battery_off_doorlight_timer = 20.0;
                        } else {
                            self.door_1_flip_flop = !self.door_1_flip_flop;
                        }
                    } else {
                        self.door_1_flip_flop = false;
                    }
                }
            }
            UsedDoorControl::NewControls { btn_gtu_door1, .. } => {
                if btn_gtu_door1.is_just_pressed() && !battery {
                    if self.door_1_flip_flop {
                        if self.battery_off_doorlight_timer <= 0.0 {
                            self.battery_off_doorlight_timer = 20.0;
                        } else {
                            self.door_1_flip_flop = !self.door_1_flip_flop;
                        }
                    } else {
                        self.door_1_flip_flop = false;
                    }
                }
            }
        }

        if km_h.abs() > 0.6 {
            self.door_1_flip_flop = false;
            self.door_4_flip_flop = false;
        }

        // Targets anpassen
        if self.door_target == DoorTarget::Open {
            self.door_1_flip_flop = false;
            self.door_1_flip_flop_last = false;
            self.door_4_flip_flop = false;
            self.door_4_flip_flop_last = false;
        }

        self.door_2_target = self.door_target;
        self.door_3_target = self.door_target;

        if self.door_1_flip_flop_last != self.door_1_flip_flop {
            self.door_1_target = if self.door_1_flip_flop {
                DoorTarget::Open
            } else {
                DoorTarget::FastClose
            };
        }

        if (!self.door_1_flip_flop
            && (self.door_1_target != DoorTarget::FastClose
                || self.door_target == DoorTarget::Open))
            || (self.door_1.state == DoorState::Closed
                && self.door_1_target == DoorTarget::FastClose)
        {
            self.door_1_target = self.door_target;
        }

        if self.door_4_flip_flop_last != self.door_4_flip_flop {
            self.door_4_target = if self.door_4_flip_flop {
                DoorTarget::Open
            } else {
                DoorTarget::FastClose
            };
        }

        if (!self.door_4_flip_flop
            && (self.door_4_target != DoorTarget::FastClose
                || self.door_target == DoorTarget::Open))
            || (self.door_4.state == DoorState::Closed
                && self.door_4_target == DoorTarget::FastClose)
        {
            self.door_4_target = self.door_target;
        }

        // Apply fuses
        if !fuse_door_1_power {
            self.door_1_target = DoorTarget::Close;
        }

        if !fuse_central_door_control {
            self.door_target = DoorTarget::Close;
            self.door_1_target = DoorTarget::Close;
            self.door_1_flip_flop = false;
            self.door_1_flip_flop_last = false;
            self.door_2_target = DoorTarget::Close;
            self.door_3_target = DoorTarget::Close;
            self.door_4_target = DoorTarget::Close;
            self.door_4_flip_flop = false;
            self.door_4_flip_flop_last = false;
        }

        self.force_close_requested =
            (door_target_last > DoorTarget::Close) && (self.door_target <= DoorTarget::Close);

        self.door_1_flip_flop_last = self.door_1_flip_flop;
        self.door_4_flip_flop_last = self.door_4_flip_flop;
    }

    pub fn on_message(&mut self, msg: Message) {
        self.stop_request.on_message(msg.clone());
        self.door_control_send_coupling.on_message(msg.clone());
        self.door_control_receive_coupling.on_message(msg.clone());
        self.doors_closed_send_coupling.on_message(msg.clone());
        self.doors_closed_receive_coupling.on_message(msg.clone());
    }
}

impl Default for Doors {
    fn default() -> Self {
        Self::new()
    }
}

struct DoorLight {
    doorlight_1: Light,
    doorlight_2: Light,
    doorlight_3: Light,
    doorlight_4: Light,
}

impl DoorLight {
    fn new() -> Self {
        Self {
            doorlight_1: Light::new(Some("L_Door_1_Tuerlicht")),
            doorlight_2: Light::new(Some("L_Door_2_Tuerlicht")),
            doorlight_3: Light::new(Some("L_Door_3_Tuerlicht")),
            doorlight_4: Light::new(Some("L_Door_4_Tuerlicht")),
        }
    }

    fn tick(
        &mut self,
        door_states: [DoorState; 4],
        battery_off_doorlight_timer: f32,
        com: &mut Com,
    ) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let permanent_voltage = com.lv.get_or(WslPermanentVoltageNorm, 0.0);
        let interior_light = com.lv.get_or(WslInteriorLight, false);
        let emergency_light = com.lv.get_or(WslEnergencyLight, false);

        // Main logic
        let doorlight_1_target = (((*door_states.first().unwrap_or(&DoorState::Closed))
            != DoorState::Closed
            && interior_light)
            || emergency_light
            || battery_off_doorlight_timer > 0.0) as u8 as f32
            * permanent_voltage;
        let doorlight_2_target = ((*door_states.get(2).unwrap_or(&DoorState::Closed))
            != DoorState::Closed
            && interior_light) as u8 as f32
            * voltage;
        let doorlight_3_target = ((*door_states.get(3).unwrap_or(&DoorState::Closed))
            != DoorState::Closed
            && interior_light) as u8 as f32
            * voltage;
        let doorlight_4_target = ((*door_states.get(4).unwrap_or(&DoorState::Closed))
            != DoorState::Closed
            && interior_light) as u8 as f32
            * voltage;

        // Assign output
        self.doorlight_1.set_brightness(doorlight_1_target);
        self.doorlight_2.set_brightness(doorlight_2_target);
        self.doorlight_3.set_brightness(doorlight_3_target);
        self.doorlight_4.set_brightness(doorlight_4_target);
    }
}
