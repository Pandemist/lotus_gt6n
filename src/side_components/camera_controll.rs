use lotus_extra::vehicle::CockpitSide;
use pandemist_vehicle_elements::{
    components::gt6n::videosystem::VideoSystemGt6n,
    elements::tech::{
        buttons::PushButton,
        switches::{StepSwitch, SwitchEventAction},
    },
    management::{communicator::Com, enums::general_enums::CabActivState},
};

use crate::general::local_values::{WslCabState, WslLowVoltageNorm};

pub struct CameraControll {
    a_sw_brightness: StepSwitch,
    a_sw_recording: StepSwitch,
    a_btn_cam_1: PushButton,
    a_btn_cam_2: PushButton,

    a_cam_controller: VideoSystemGt6n,
}

impl CameraControll {
    pub fn new() -> Self {
        Self {
            a_sw_brightness: StepSwitch::builder(
                "AV_A_Sw_Videomonitor_Helligkeit",
                Some(CockpitSide::A),
            )
            .event("Videomonitor_Helligkeit_Plus", SwitchEventAction::Plus)
            .event("Videomonitor_Helligkeit_Minus", SwitchEventAction::Minus)
            .snd_default_plus("Snd_A_Switch")
            .snd_default_minus("Snd_A_Switch")
            .min(-1)
            .max(1)
            .min_spring()
            .max_spring()
            .build(),
            a_sw_recording: StepSwitch::builder("AV_A_Sw_Video_DS_AS", Some(CockpitSide::A))
                .event("Videomonitor_DS_AS_Plus", SwitchEventAction::Plus)
                .event("Videomonitor_DS_AS_Minus", SwitchEventAction::Minus)
                .snd_default_plus("Snd_A_Switch")
                .snd_default_minus("Snd_A_Switch")
                .min(-1)
                .max(1)
                .min_spring()
                .max_spring()
                .build(),
            a_btn_cam_1: PushButton::builder("AV_A_Btn_1", "Btn_1", Some(CockpitSide::A))
                .snd_press("Snd_A_BtnDn")
                .snd_release("Snd_A_BtnUp")
                .build(),
            a_btn_cam_2: PushButton::builder("AV_A_Btn_2", "Btn_2", Some(CockpitSide::A))
                .snd_press("Snd_A_BtnDn")
                .snd_release("Snd_A_BtnUp")
                .build(),
            a_cam_controller: VideoSystemGt6n::new("LM_A_Kammera_Gruen", "LM_A_Kammera_Rot"),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        // Read local signals
        let voltage = com.lv.get_or(WslLowVoltageNorm, 0.0);
        let cab_a_activ =
            com.lv.get_or(WslCabState(0), CabActivState::default()) > CabActivState::Off;

        // Read fuses

        // Input from key events
        self.a_sw_brightness.tick();
        self.a_sw_recording.tick();
        self.a_btn_cam_1.tick();
        self.a_btn_cam_2.tick();

        // Input - Signale

        // Main logic
        self.a_cam_controller.tick(cab_a_activ, voltage);

        // Assign output
    }
}

impl Default for CameraControll {
    fn default() -> Self {
        Self::new()
    }
}
