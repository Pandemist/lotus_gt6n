use pandemist_vehicle_elements::management::{
    enums::{
        general_enums::TrainFormationSwitch, state_enums::ChangedState,
        traction_enums::DirectionOfDriving,
    },
    structs::general_structs::TrainActivState,
};
use typedmap::TypedMapKey;

//==========================================================================
// Structural
//==========================================================================

/// Represents the blocked state of a WSL (Wayside Station Logic) flap.
///
/// The `usize` parameter identifies the specific flap instance.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslFlapBlocked(pub usize);

impl TypedMapKey for WslFlapBlocked {
    type Value = bool;
}

/// Represents the active state of a WSL wheelchair accessibility helper.
///
/// The `usize` parameter identifies the specific wheelchair helper instance.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslWheelchairHelperActive(pub usize);

impl TypedMapKey for WslWheelchairHelperActive {
    type Value = bool;
}

//==========================================================================
// Coupling
//==========================================================================

/// Represents the electrical coupling state between train cars.
///
/// The `usize` parameter identifies the specific coupling instance.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslElectricCoupled(pub usize);

impl TypedMapKey for WslElectricCoupled {
    type Value = bool;
}

//==========================================================================
// Car electrics
//==========================================================================

/// Represents the state of the main battery switch.
///
/// This key tracks changes to the battery main switch state.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslBatteryMainSwitch;

impl TypedMapKey for WslBatteryMainSwitch {
    type Value = ChangedState;
}

/// Represents the normalized low voltage reading.
///
/// Values are typically expressed as a ratio or percentage of nominal voltage.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslLowVoltageNorm;

impl TypedMapKey for WslLowVoltageNorm {
    type Value = f32;
}

/// Represents the normalized permanent voltage reading.
///
/// This tracks the permanent power supply voltage as a normalized value.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslPermanentVoltageNorm;

impl TypedMapKey for WslPermanentVoltageNorm {
    type Value = f32;
}

/// Represents the normalized converter voltage reading.
///
/// This tracks the voltage output from power converters.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslConverterVoltageNorm;

impl TypedMapKey for WslConverterVoltageNorm {
    type Value = f32;
}

/// Represents the normalized traction voltage reading.
///
/// This tracks the voltage supplied to traction motors.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslTractionVoltageNorm;

impl TypedMapKey for WslTractionVoltageNorm {
    type Value = f32;
}

//==========================================================================
// Lighting
//==========================================================================

/// Represents the brightness level of cab indicators.
///
/// The `usize` parameter identifies the specific cab instance.
/// Brightness is typically expressed as a floating-point value between 0.0 and 1.0.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslCabIndicatorBrightness(pub usize);

impl TypedMapKey for WslCabIndicatorBrightness {
    type Value = f32;
}

/// Represents the state of a light test function.
///
/// The `usize` parameter identifies the specific light test instance.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslLighttest(pub usize);

impl TypedMapKey for WslLighttest {
    type Value = bool;
}

/// Represents the state of interior lighting.
///
/// `true` indicates the interior lights are on, `false` indicates they are off.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslInteriorLight;

impl TypedMapKey for WslInteriorLight {
    type Value = bool;
}

/// Represents the state of emergency lighting.
///
/// `true` indicates emergency lights are active, `false` indicates they are inactive.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslEnergencyLight;

impl TypedMapKey for WslEnergencyLight {
    type Value = bool;
}

/// Represents the state of the brake lights.
///
/// `true` indicates brake lights are active, `false` indicates they are inactive.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslBrakelight(pub usize);

impl TypedMapKey for WslBrakelight {
    type Value = bool;
}

//==========================================================================
// Doors
//==========================================================================

/// Represents whether all doors are closed.
///
/// `true` indicates all doors are closed and locked, `false` indicates at least one door is open.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslDoorsClosed;

impl TypedMapKey for WslDoorsClosed {
    type Value = bool;
}

//==========================================================================
// Traction
//==========================================================================

/// Represents the current driving direction of the train.
///
/// This indicates whether the train is moving forward, backward, or is stationary.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslDirectionOfDriving;

impl TypedMapKey for WslDirectionOfDriving {
    type Value = DirectionOfDriving;
}

/// Represents the current speed reading in kilometers per hour.
///
/// This is the speed as displayed on the train's speedometer.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslSpeedometerKmh;

impl TypedMapKey for WslSpeedometerKmh {
    type Value = f32;
}

/// Represents the state of the rail brake warning bell.
///
/// `true` indicates the bell is active/ringing, `false` indicates it is silent.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslExtraBellTarget;

impl TypedMapKey for WslExtraBellTarget {
    type Value = bool;
}

/// Represents the state of emergency brakes.
///
/// `true` indicates emergency brakes are engaged, `false` indicates they are not active.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslEmergencyBrakes;

impl TypedMapKey for WslEmergencyBrakes {
    type Value = bool;
}

/// Represents the target traction value.
///
/// This is the desired traction force or power level, typically as a normalized value.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslTractionTarget;

impl TypedMapKey for WslTractionTarget {
    type Value = f32;
}

/// Represents the sanding switch by switch.
///
/// This is the desired sanding target by cabin switch.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslSandSwitchTarget(pub usize);

impl TypedMapKey for WslSandSwitchTarget {
    type Value = bool;
}

//==========================================================================
// Driver's cab status (full Train)
//==========================================================================

/// Represents the activation state of a driver's cab in all cars.
///
/// The `usize` parameter identifies the specific cab instance.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslTrainState;

impl TypedMapKey for WslTrainState {
    type Value = TrainActivState;
}

/// Indicates whether the driver's cab has been dismantled but the driver has not yet left it (door 1 activated).
///
/// The `usize` parameter identifies the specific cab instance.
/// `true` indicates the cab is turned off but still connected/present.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslCabOffButStillThere(pub usize);

impl TypedMapKey for WslCabOffButStillThere {
    type Value = bool;
}

/// Represents the position of the train formation switch.
///
/// The `usize` parameter identifies the specific switch instance.
/// This switch determines how multiple train units are configured together.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslTrainFormationSwitch(pub usize);

impl TypedMapKey for WslTrainFormationSwitch {
    type Value = TrainFormationSwitch;
}

/// Represents the state of a workshop maintenance key.
///
/// The `usize` parameter identifies the specific key instance.
/// `true` indicates the workshop key is active/inserted.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct WslWorkshopKey(pub usize);

impl TypedMapKey for WslWorkshopKey {
    type Value = bool;
}
