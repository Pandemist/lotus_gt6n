use lotus_extra::vehicle::CockpitSide;
use lotus_script::{message::Coupling, prelude::Message};
use pandemist_vehicle_elements::{
    components::general::coupler::HandCoupler,
    elements::tech::switches::{StepSwitch, SwitchEventAction},
    management::{
        communicator::Com,
        enums::general_enums::{CabActivState, TrainFormationSwitch},
        structs::general_structs::TrainActivState,
        trainbus::EcouplerSender,
    },
    messages::{
        diagnostic_messages::{DiagnosticFaultKind, DiagnosticMessageSender},
        gt6n_coupling_messages,
    },
};

use crate::general::local_values::{WslElectricCoupled, WslTrainFormationSwitch, WslTrainState};

pub struct VehicleCoupling {
    mms_fault_sender: DiagnosticMessageSender,

    el_sender: EcouplerSender,

    coupler_0: HandCoupler,
    coupler_1: HandCoupler,

    bag_a: gt6n_coupling_messages::BagReader,
    bag_b: gt6n_coupling_messages::BagReader,

    a_sw_train_formation_switch: StepSwitch,
}

impl VehicleCoupling {
    pub fn new() -> Self {
        Self {
            mms_fault_sender: DiagnosticMessageSender::default(),

            el_sender: EcouplerSender::default(),

            coupler_0: HandCoupler::new(0, Some(CockpitSide::A), Coupling::Front, 0.3, -0.002),
            coupler_1: HandCoupler::new(1, Some(CockpitSide::B), Coupling::Rear, 0.3, -0.002),

            bag_a: gt6n_coupling_messages::BagReader::new(Coupling::Front),
            bag_b: gt6n_coupling_messages::BagReader::new(Coupling::Rear),

            a_sw_train_formation_switch: StepSwitch::builder(
                "AV_A_Sw_Zugbildungsschalter",
                Some(CockpitSide::A),
            )
            .event("Zugbildungsschalter_Plus", SwitchEventAction::Plus)
            .event("Zugbildungsschalter_Minus", SwitchEventAction::Minus)
            .snd_default_plus("Snd_A_Switch")
            .snd_default_minus("Snd_A_Switch")
            .min(-1)
            .max(1)
            .init(-(Coupling::Rear.is_coupled() as u8 as i32))
            .build(),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let cab_a_activ = com
            .lv
            .get_or(WslTrainState, TrainActivState::default())
            .cab_a
            > CabActivState::Off;

        // Read fuses

        // Input from key events
        self.coupler_0.tick(self.bag_a.value);
        self.coupler_1.tick(self.bag_b.value);

        self.a_sw_train_formation_switch.tick();

        // Input - Signale

        // Main logic
        if !self.coupler_0.mech_coupled() {
            self.bag_a.value = false;
        }
        if !self.coupler_1.mech_coupled() {
            self.bag_b.value = false;
        }

        //gt6n oder Gt6o
        com.lv.set(
            WslTrainFormationSwitch(0),
            match self.a_sw_train_formation_switch.value(true) {
                -1 => TrainFormationSwitch::TractionLeader,
                1 => TrainFormationSwitch::Following,
                _ => TrainFormationSwitch::Leading,
            },
        );

        // Assign output
        self.el_sender.update_front(self.coupler_0.el_coupled());
        self.el_sender.update_rear(self.coupler_1.el_coupled());

        com.lv
            .set(WslElectricCoupled(0), self.coupler_0.el_coupled());
        com.lv
            .set(WslElectricCoupled(1), self.coupler_1.el_coupled());

        //===============================================================
        // MMS communication
        //===============================================================

        let zugbildungsfehler = (cab_a_activ
            && self.coupler_1.el_coupled()
            && self.a_sw_train_formation_switch.value(true) != -1)
            || (cab_a_activ
                && !self.coupler_1.el_coupled()
                && self.a_sw_train_formation_switch.value(true) == -1);

        self.mms_fault_sender.send(
            DiagnosticFaultKind::ZugbildungsFehlerA,
            zugbildungsfehler,
            Some(CockpitSide::A),
        );
    }

    pub fn on_message(&mut self, msg: Message) {
        self.bag_a.on_message(msg.clone());
        self.bag_b.on_message(msg.clone());
    }
}

impl Default for VehicleCoupling {
    fn default() -> Self {
        Self::new()
    }
}
