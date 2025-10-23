use lotus_script::time::delta;
use pandemist_vehicle_elements::{
    api::axis::ApiRailAxis,
    management::{communicator::Com, enums::traction_enums::DirectionOfDriving},
    messages::diagnostic_messages::DiagnosticSlipSender,
};

use crate::general::local_values::{WslDirectionOfDriving, WslSpeedometerKmh};

const SLIP_BOUND_MPS: f32 = 1.0;
const SLIDE_BOUND_MPS: f32 = 1.0;
const SLIP_DETECTION_INTERVAL: f32 = 0.5;
const SLIDE_RAILBRAKE_WAITTIME_MIN: f32 = 0.5;
const SLIDE_RAILBRAKE_WAITTIME_ADD: f32 = 0.5;

pub struct AntiSlipAntiSlideProtectionUnit {
    mms_slip_sender: DiagnosticSlipSender,

    // Gleitschutz
    pub anti_slide_active: bool,
    pub anti_slide_railbrake: bool,
    // Schleuderschutz
    pre_anti_slip_active: bool,
    pub anti_slip_active: bool,

    anti_slide_timer: f32,
    anti_slide_traction_target: f32,
    timer: f32,
}

impl AntiSlipAntiSlideProtectionUnit {
    pub fn new() -> Self {
        Self {
            mms_slip_sender: DiagnosticSlipSender::default(),

            anti_slide_active: false,
            anti_slide_railbrake: false,
            anti_slip_active: false,
            pre_anti_slip_active: false,

            anti_slide_timer: 0.0,
            anti_slide_traction_target: 0.0,
            timer: 0.0,
        }
    }

    pub fn tick(
        &mut self,
        traction_target: f32,
        fast_emergency_brake: bool,
        anti_slip_ueberbrueckt: bool,
        com: &mut Com,
    ) {
        // Read local signals
        let km_h = com.lv.get_or(WslSpeedometerKmh, 0.0);
        let direction_of_driving = com
            .lv
            .get_or(WslDirectionOfDriving, DirectionOfDriving::default());

        // Main logic
        let api_axis_0_0 = ApiRailAxis::new(0, 0);
        let api_axis_0_1 = ApiRailAxis::new(1, 0);
        let api_axis_2_0 = ApiRailAxis::new(0, 2);
        let api_axis_2_1 = ApiRailAxis::new(1, 2);

        let anti_slip_new = if fast_emergency_brake || km_h.abs() < 0.1 {
            false
        } else if direction_of_driving.forward {
            api_axis_0_1.speed_mps().abs() > (api_axis_0_0.speed_mps().abs() + SLIP_BOUND_MPS)
        } else if direction_of_driving.backward {
            api_axis_2_0.speed_mps().abs() > (api_axis_2_1.speed_mps().abs() + SLIP_BOUND_MPS)
        } else {
            false
        };

        let anti_slide_new = if fast_emergency_brake || km_h.abs() < 0.1 {
            false
        } else if self.anti_slide_active
            && traction_target < (self.anti_slide_traction_target + 0.05)
        {
            true
        } else {
            let tmp_anti_slide_new = if direction_of_driving.forward {
                api_axis_0_1.speed_mps().abs() < (api_axis_0_0.speed_mps().abs() + SLIDE_BOUND_MPS)
            } else if direction_of_driving.backward {
                api_axis_2_0.speed_mps().abs() < (api_axis_2_1.speed_mps().abs() + SLIDE_BOUND_MPS)
            } else {
                false
            };
            if tmp_anti_slide_new {
                self.anti_slide_traction_target = traction_target;
            }
            tmp_anti_slide_new
        };

        // Anti Slide
        if self.timer > 0.0 {
            self.timer -= delta();
        } else if anti_slide_new != self.anti_slide_active
            || anti_slip_new != self.pre_anti_slip_active
        {
            self.anti_slide_active = anti_slide_new;
            self.pre_anti_slip_active = anti_slip_new;
            self.timer = SLIP_DETECTION_INTERVAL;
        }

        self.anti_slip_active = self.pre_anti_slip_active && !anti_slip_ueberbrueckt;

        self.mms_slip_sender.send(self.anti_slip_active);

        // Anti Slip
        if self.anti_slide_active {
            self.anti_slide_timer += delta();
        } else {
            self.anti_slide_timer = 0.0;
        }

        // Assign output
        self.anti_slide_railbrake = self.anti_slide_active
            && traction_target < 0.0
            && (self.anti_slide_timer
                > SLIDE_RAILBRAKE_WAITTIME_MIN
                    + SLIDE_RAILBRAKE_WAITTIME_ADD * (traction_target + 1.0));

        // TODO
        self.anti_slide_active = false;
        self.anti_slide_railbrake = false;
        self.anti_slip_active = false;
        self.pre_anti_slip_active = false;
    }
}

impl Default for AntiSlipAntiSlideProtectionUnit {
    fn default() -> Self {
        Self::new()
    }
}
