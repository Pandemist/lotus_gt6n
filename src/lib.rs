/*#![warn(
    clippy::all,
//    clippy::restriction,
    clippy::pedantic,
//    clippy::nursery,
    clippy::cargo
)]*/

use dom_components::{
    coupling::VehicleCoupling, doors::Doors, electric::Electric, lights::Lights, traction::Traction,
};
use general::setup::initialize;
use lotus_script::{prelude::Message, rand::random_seed, script, Script};
use pandemist_vehicle_elements::{
    api::vehicle_infos::veh_number,
    elements::tech::key_switch::KeyDepot,
    management::{communicator::Com, trainbus::TrainBusManager},
    messages::diagnostic_messages::MsgDiagnosticAntiSlideOverride,
};
use side_components::{
    bell::Bell, camera_controll::CameraControll, companion_call::CompanionCall,
    heater_ventilation::HeatingVentilation, intercoms_em_brake::IntercomsEmBrake, mirrors::Mirrors,
    moveables::Moveables, simple_components::SimpleComponents, wipers::Wipers,
};

pub mod dom_components;
pub mod general;
pub mod side_components;

script!(Gt6n);

pub struct Gt6n {
    com: Com,
    zugbus: TrainBusManager,
    // Dominant Components
    coupling: VehicleCoupling,
    doors: Doors,
    electric: Electric,
    lights: Lights,
    traction: Traction,
    // Subdominant Components
    bell: Bell,
    camera_conroll: CameraControll,
    companion_call: CompanionCall,
    simple_components: SimpleComponents,
    intercoms_em_brake: IntercomsEmBrake,
    heating_ventilation: HeatingVentilation,
    mirrors: Mirrors,
    moveables: Moveables,
    wiper: Wipers,
}

impl Default for Gt6n {
    fn default() -> Self {
        random_seed();

        let mut com = Com::new();

        let driver_key = KeyDepot::new("DriverKey".to_string());
        driver_key.put_in();

        let workshop_key = KeyDepot::new("WorkshopKey".to_string());
        workshop_key.put_in();

        let moveables = Moveables::new(driver_key.clone());

        let coupling = VehicleCoupling::new();
        let doors = Doors::new();
        let electric = Electric::new(driver_key.clone());
        let lights = Lights::new(driver_key.clone());
        let traction = Traction::new(driver_key.clone());

        let bell = Bell::new();
        let camera_conroll = CameraControll::new();
        let companion_call = CompanionCall::new();
        let simple_components = SimpleComponents::new(workshop_key.clone());
        let intercoms_em_brake = IntercomsEmBrake::new();
        let heating_ventilation = HeatingVentilation::new();
        let mirrors = Mirrors::new();
        let wiper = Wipers::new();

        initialize(&mut com);

        let zugbus = TrainBusManager::new(veh_number(), None);

        Self {
            com,
            zugbus,
            // Dominant Components
            coupling,
            doors,
            electric,
            lights,
            traction,
            // Subdominant Components
            bell,
            camera_conroll,
            companion_call,
            simple_components,
            intercoms_em_brake,
            heating_ventilation,
            mirrors,
            moveables,
            wiper,
        }
    }
}

impl Script for Gt6n {
    fn tick(&mut self) {
        self.moveables.tick(&mut self.com);
        // Dominant Components
        self.coupling.tick(&mut self.com);
        self.doors.tick(&mut self.com);
        self.electric.tick(&mut self.com);
        self.lights.tick(&mut self.com);
        self.traction.tick(&mut self.com);
        // Subdominant Components
        self.bell.tick(&mut self.com);
        self.camera_conroll.tick(&mut self.com);
        self.companion_call.tick(&mut self.com);
        self.simple_components.tick(&mut self.com);
        self.intercoms_em_brake.tick(&mut self.com);
        self.heating_ventilation.tick(&mut self.com);
        self.mirrors.tick(&mut self.com);
        self.wiper.tick(&mut self.com);
    }

    fn on_message(&mut self, msg: Message) {
        self.zugbus.on_message(msg.clone());

        self.coupling.on_message(msg.clone());
        self.doors.on_message(msg.clone());
        self.electric.on_message(msg.clone());
        self.lights.on_message(msg.clone());
        self.traction.on_message(msg.clone());

        self.companion_call.on_message(msg.clone());
        self.intercoms_em_brake.on_message(msg.clone());
        self.simple_components.on_message(msg.clone());

        msg.handle::<MsgDiagnosticAntiSlideOverride>(|m| {
            self.traction.anti_slip_override = m.value;
            Ok(())
        })
        .expect("MsgDiagnosticAntiSlideOverride: message handle failed");
    }
}
