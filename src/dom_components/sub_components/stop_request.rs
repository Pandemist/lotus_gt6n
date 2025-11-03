use lotus_extra::{messages::std::StopRequestSender, vehicle::CockpitSide};
use lotus_script::prelude::Message;
use pandemist_vehicle_elements::{
    api::light::Light,
    components::doors::door_buttons::{RedGreenDuoBtn, RedGreenOutBtn, SimpleInBtn},
    elements::tech::buttons::PushButton,
    management::{
        communicator::Com,
        enums::{
            door_enums::{DoorState, DoorTarget},
            general_enums::CabActivState,
            state_enums::ChangedState,
            traction_enums::DirectionOfDriving,
        },
        structs::general_structs::TrainActivState,
    },
    messages::{
        coupling_handler::UniversalCouplingLine,
        gt6n_coupling_messages::{CouplerBuggyReqest, CouplerBuggyReset, CouplerStopRequest},
    },
};

use crate::general::{
    local_values::{
        WslBatteryMainSwitch, WslCabIndicatorBrightness, WslDirectionOfDriving, WslLighttest,
        WslLowVoltageNorm, WslTrainState,
    },
    setup::{const_veh_variant, FahrzeugVariante},
};

pub struct StopRequest {
    stop_request_sender: StopRequestSender,

    // Außentaster
    door_btn_1: RedGreenOutBtn,
    door_btn_2: RedGreenOutBtn,
    door_btn_3: RedGreenOutBtn,
    door_btn_4: RedGreenOutBtn,

    buggy_wheelchair_out_btn_1a: RedGreenDuoBtn,
    buggy_wheelchair_out_btn_1b: RedGreenDuoBtn,
    buggy_out_btn_2: RedGreenOutBtn,
    buggy_out_btn_4a: RedGreenOutBtn,
    buggy_out_btn_4b: RedGreenOutBtn,

    // Innentaster
    stop_request_btn_1: SimpleInBtn,
    stop_request_btn_2: SimpleInBtn,
    stop_request_btn_3: SimpleInBtn,
    stop_request_btn_4: SimpleInBtn,

    buggy_wheelchair_in_btn_1a: RedGreenDuoBtn,
    buggy_wheelchair_in_btn_1b: RedGreenDuoBtn,

    buggy_handrail_1: SimpleInBtn,
    buggy_handrail_2: SimpleInBtn,
    buggy_handrail_4: SimpleInBtn,

    // Zustände
    stop_request_coupling: UniversalCouplingLine<bool, CouplerStopRequest>,
    buggy_signal_coupling: UniversalCouplingLine<bool, CouplerBuggyReqest>,
    buggy_reset_coupling: UniversalCouplingLine<bool, CouplerBuggyReset>,

    stop_request: bool,
    stop_request_1: bool,
    stop_request_2: bool,
    stop_request_3: bool,
    stop_request_4: bool,

    buggy: bool,
    buggy_1: bool,
    buggy_2: bool,
    buggy_4: bool,

    wheelchair: bool,
    wheelchair_1: bool,

    // Output
    pub request_1: bool,
    pub request_2: bool,
    pub request_3: bool,
    pub request_4: bool,

    pub force_open_1: bool,
    pub force_open_2: bool,
    pub force_open_3: bool,
    pub force_open_4: bool,

    pub lift_allowed: bool,

    // Spezifisch
    a_btn_buggy_reset: PushButton,
    a_btn_wheelchair_reset: PushButton,

    a_lm_stop_request: Light,
    a_lm_buggy: Light,
    a_lm_wheelchair: Light,
}

impl StopRequest {
    pub fn new() -> Self {
        Self {
            stop_request_sender: StopRequestSender::default(),

            door_btn_1: RedGreenOutBtn::new(
                None,
                "Tuertaster_1",
                "L_Doors_Released_1",
                "L_Door_1_Red",
                vec![0],
            ),
            door_btn_2: RedGreenOutBtn::new(
                None,
                "Tuertaster_2",
                "L_Doors_Released",
                "L_Door_2_Red",
                vec![1],
            ),
            door_btn_3: RedGreenOutBtn::new(
                None,
                "Tuertaster_3",
                "L_Doors_Released",
                "L_Door_3_Red",
                vec![2],
            ),
            door_btn_4: RedGreenOutBtn::new(
                None,
                "Tuertaster_4",
                "L_Doors_Released",
                "L_Door_4_Red",
                vec![3],
            ),
            buggy_wheelchair_out_btn_1a: RedGreenDuoBtn::new(
                None,
                "KiWa_1a",
                "Rolli_1a",
                "L_Doors_Released_RolliKiWa_1",
                "L_Rollitaster_1a_pressed",
            ),
            buggy_wheelchair_out_btn_1b: RedGreenDuoBtn::new(
                None,
                "KiWa_1b",
                "Rolli_1b",
                "L_Doors_Released_RolliKiWa_1",
                "L_Rollitaster_1b_pressed",
            ),
            buggy_out_btn_2: RedGreenOutBtn::new(
                None,
                "KiWa_2",
                "L_Doors_Released_RolliKiWa",
                "L_Rollitaster_2_pressed",
                vec![],
            ),
            buggy_out_btn_4a: RedGreenOutBtn::new(
                None,
                "KiWa_4a",
                "L_Doors_Released_RolliKiWa",
                "L_Rollitaster_4a_pressed",
                vec![],
            ),
            buggy_out_btn_4b: RedGreenOutBtn::new(
                None,
                "KiWa_4b",
                "L_Doors_Released_RolliKiWa",
                "L_Rollitaster_4b_pressed",
                vec![],
            ),
            stop_request_btn_1: SimpleInBtn::new(None, "Haltewunsch_1", vec![0]),
            stop_request_btn_2: SimpleInBtn::new(None, "Haltewunsch_2", vec![1]),
            stop_request_btn_3: SimpleInBtn::new(None, "Haltewunsch_3", vec![2]),
            stop_request_btn_4: SimpleInBtn::new(None, "Haltewunsch_4", vec![3]),
            buggy_wheelchair_in_btn_1a: RedGreenDuoBtn::new(
                None,
                "KiWa_in_1a",
                "Rolli_in_1a",
                "L_Doors_Released_RolliKiWa_1",
                "L_Rollitaster_in_1a_pressed",
            ),
            buggy_wheelchair_in_btn_1b: RedGreenDuoBtn::new(
                None,
                "KiWa_in_1b",
                "Rolli_in_1b",
                "L_Doors_Released_RolliKiWa_1",
                "L_Rollitaster_in_1b_pressed",
            ),
            buggy_handrail_1: SimpleInBtn::new(None, "KiWa_Stange_1", vec![]),
            buggy_handrail_2: SimpleInBtn::new(None, "KiWa_Stange_2", vec![]),
            buggy_handrail_4: SimpleInBtn::new(None, "KiWa_Stange_4", vec![]),

            a_btn_buggy_reset: PushButton::builder(
                "AV_A_Btn_Kinderwagen_Reset",
                "Reset_Kinderwagen",
                Some(CockpitSide::A),
            )
            .snd_press("Snd_A_BtnDn")
            .snd_release("Snd_A_BtnUp")
            .build(),
            a_btn_wheelchair_reset: {
                if const_veh_variant() != FahrzeugVariante::Gt6u {
                    PushButton::builder(
                        "AV_A_Btn_Rollstuhl_Reset",
                        "Reset_Rollstuhl",
                        Some(CockpitSide::A),
                    )
                    .snd_press("Snd_A_BtnDn")
                    .snd_release("Snd_A_BtnUp")
                    .build()
                } else {
                    PushButton::builder_time_till_hold(
                        0.5,
                        "AV_A_Btn_Rollstuhl_Reset",
                        "Reset_Rollstuhl",
                        Some(CockpitSide::A),
                    )
                    .snd_press("Snd_A_BtnDn")
                    .snd_release("Snd_A_BtnUp")
                    .build()
                }
            },
            a_lm_stop_request: Light::new(Some("LM_A_Haltewunsch")),
            a_lm_buggy: Light::new(Some("LM_A_Kinderwagen")),
            a_lm_wheelchair: Light::new(Some("LM_A_Rollstuhl")),

            stop_request_coupling: UniversalCouplingLine::new(CouplerStopRequest {}, (true, true)),
            buggy_signal_coupling: UniversalCouplingLine::new(CouplerBuggyReqest {}, (true, true)),
            buggy_reset_coupling: UniversalCouplingLine::new(CouplerBuggyReset {}, (true, true)),

            stop_request: false,
            stop_request_1: false,
            stop_request_2: false,
            stop_request_3: false,
            stop_request_4: false,

            buggy: false,
            buggy_1: false,
            buggy_2: false,
            buggy_4: false,

            wheelchair: false,
            wheelchair_1: false,

            request_1: false,
            request_2: false,
            request_3: false,
            request_4: false,

            force_open_1: false,
            force_open_2: false,
            force_open_3: false,
            force_open_4: false,

            lift_allowed: false,
        }
    }

    pub fn tick(
        &mut self,
        door_states: [DoorState; 4],
        door_target: DoorTarget,
        lift_request: bool,
        com: &mut Com,
    ) {
        // Read local signals
        let battery_switch =
            com.lv.get_or(WslBatteryMainSwitch, ChangedState::default()) >= ChangedState::JustOn;
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let released = door_target > DoorTarget::Close;
        let cab_a_activ = com
            .lv
            .get_or(WslTrainState, TrainActivState::default())
            .cab_a
            > CabActivState::Off;
        //let cab_a_runmode =
        //    com.lv.get_or(WslTrainState, TrainActivState::default()).cab_a > CabActivState::Star;
        let light_test = com.lv.get_or(WslLighttest(0), false);
        let cab_indicator_light_level = com.lv.get_or(WslCabIndicatorBrightness(0), 1.0);
        let direction_of_driving = com
            .lv
            .get_or(WslDirectionOfDriving, DirectionOfDriving::default());

        // Read fuses
        let allowed_buggy = battery_switch
            && direction_of_driving.is_one()
            && com.fuse.is_on("ZentraleTuersteuerung");
        let allowed_wheelchair = battery_switch
            && direction_of_driving.is_one()
            && com.fuse.is_on("ZentraleTuersteuerung");

        // Rampe reservieren, nur wenn die Türen erlaubt sind
        self.lift_allowed = self.lift_allowed || lift_request;

        // Input from key events
        self.a_btn_buggy_reset.tick();
        self.a_btn_wheelchair_reset.tick();

        self.buggy_signal_coupling
            .update_local(self.a_btn_buggy_reset.value(cab_a_activ && released));

        if self.buggy_signal_coupling.get_value() {
            self.buggy_1 = false;
            self.buggy_2 = false;
            self.buggy_4 = false;
        }

        if self
            .a_btn_wheelchair_reset
            .value(cab_a_activ && allowed_wheelchair)
        {
            self.wheelchair_1 = false;

            if !lift_request {
                self.lift_allowed = false;
            }
        }

        // Input - Signale

        // Main logic

        //ticks
        self.buggy_wheelchair_out_btn_1a.tick(released, false);
        self.buggy_wheelchair_out_btn_1b.tick(released, false);
        self.buggy_out_btn_4a.tick(released, false);
        self.buggy_out_btn_4b.tick(released, false);
        self.buggy_wheelchair_in_btn_1a
            .tick(self.buggy_1 || self.wheelchair_1, false);
        self.buggy_wheelchair_in_btn_1b
            .tick(self.buggy_1 || self.wheelchair_1, false);

        // Kinderwagen
        let local_buggy_1 = self.buggy_wheelchair_out_btn_1a.value_buggy()
            || self.buggy_wheelchair_out_btn_1b.value_buggy()
            || self.buggy_wheelchair_in_btn_1a.value_buggy()
            || self.buggy_wheelchair_in_btn_1b.value_buggy()
            || self.buggy_handrail_1.tick();

        let local_buggy_2 =
            self.buggy_out_btn_2.tick(released, false) || self.buggy_handrail_2.tick();

        let local_buggy_4 = self.buggy_out_btn_4a.tick(released, false)
            || self.buggy_out_btn_4b.tick(released, false)
            || self.buggy_handrail_4.tick();

        self.buggy_1 = allowed_buggy && (self.buggy_1 || local_buggy_1);
        self.buggy_2 = allowed_buggy && (self.buggy_2 || local_buggy_2);
        self.buggy_4 = allowed_buggy && (self.buggy_4 || local_buggy_4);

        // Signal durch die Zugstuerleitung geben
        self.buggy_signal_coupling
            .update_local(self.buggy_1 || self.buggy_2 || self.buggy_4);
        self.buggy = self.buggy_signal_coupling.get_value();

        // Rollstuhl
        // Rollstuhl Signal darf es nur im ersten Wagen geben, abfallen lassen, wenn nicht erster Wagen
        let local_wheelchair_1 = cab_a_activ
            && (self.buggy_wheelchair_out_btn_1a.value_wheelchair()
                || self.buggy_wheelchair_out_btn_1b.value_wheelchair()
                || self.buggy_wheelchair_in_btn_1a.value_wheelchair()
                || self.buggy_wheelchair_in_btn_1b.value_wheelchair());

        self.wheelchair_1 = allowed_wheelchair && (self.wheelchair_1 || local_wheelchair_1);

        self.wheelchair = self.wheelchair_1 || self.lift_allowed;

        // Stopreqest

        let stop_request_btn_1 = self.stop_request_btn_1.tick();
        let stop_request_btn_2 = self.stop_request_btn_2.tick();
        let stop_request_btn_3 = self.stop_request_btn_3.tick();
        let stop_request_btn_4 = self.stop_request_btn_4.tick();

        self.stop_request_1 = allowed_buggy
            && (*door_states.first().unwrap_or(&DoorState::Closed) == DoorState::Closed)
            && (self.stop_request_1 || stop_request_btn_1 || local_buggy_1 || local_wheelchair_1);

        self.stop_request_2 = allowed_buggy
            && (*door_states.get(1).unwrap_or(&DoorState::Closed) == DoorState::Closed)
            && (self.stop_request_2 || stop_request_btn_2 || local_buggy_2);

        self.stop_request_3 = allowed_buggy
            && (*door_states.get(2).unwrap_or(&DoorState::Closed) == DoorState::Closed)
            && (self.stop_request_3 || stop_request_btn_3);

        self.stop_request_4 = allowed_buggy
            && (*door_states.get(3).unwrap_or(&DoorState::Closed) == DoorState::Closed)
            && (self.stop_request_4 || stop_request_btn_4 || local_buggy_4);

        // Signal durch die Zugsteuerleitung geben
        self.stop_request_coupling.update_local(
            self.stop_request_1
                || self.stop_request_2
                || self.stop_request_3
                || self.stop_request_4,
        );
        self.stop_request = self.stop_request_coupling.get_value();

        // Assign output
        let door_btn_1 = self.door_btn_1.tick(released, false);
        let door_btn_2 = self.door_btn_2.tick(released, false);
        let door_btn_3 = self.door_btn_3.tick(released, false);
        let door_btn_4 = self.door_btn_4.tick(released, false);

        self.request_1 = self.stop_request_1 || door_btn_1;
        self.request_2 = self.stop_request_2 || door_btn_2;
        self.request_3 = self.stop_request_3 || door_btn_3;
        self.request_4 = self.stop_request_4 || door_btn_4;

        self.force_open_1 = self.buggy_1 || self.wheelchair_1;
        self.force_open_2 = self.buggy_2;
        self.force_open_3 = false;
        self.force_open_4 = self.buggy_4;

        self.stop_request_sender.send(self.stop_request);

        self.a_lm_stop_request.set_brightness(
            voltage
                * cab_indicator_light_level
                * ((cab_a_activ && self.stop_request) || light_test) as u8 as f32,
        );
        self.a_lm_buggy.set_brightness(
            voltage
                * cab_indicator_light_level
                * ((cab_a_activ && self.buggy) || light_test) as u8 as f32,
        );
        self.a_lm_wheelchair.set_brightness(
            voltage
                * cab_indicator_light_level
                * ((cab_a_activ && self.wheelchair) || light_test) as u8 as f32,
        );
    }

    pub fn on_message(&mut self, msg: Message) {
        self.buggy_reset_coupling.on_message(msg.clone());
        self.stop_request_coupling.on_message(msg.clone());
        self.buggy_signal_coupling.on_message(msg.clone());
    }
}

impl Default for StopRequest {
    fn default() -> Self {
        Self::new()
    }
}
