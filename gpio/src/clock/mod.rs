//! Clock manager module.

pub mod raw;

use std::fmt::Debug;
use crate::{GpioError, GpioResult};

/// ClockDriver trait defines the interface for clock drivers in the GPIO subsystem.
/// 
/// For the raw implementation and device-level documentation, see [raw::RawClockDriver].
pub trait ClockDriver: Debug {
    /// Checks whether the clock is currently enabled.
    fn enabled(&self) -> GpioResult<bool>;
    /// Sets the enabled state of the clock.
    fn set_enabled(&mut self, enabled: bool) -> GpioResult<()>;

    /// Gets the current mash mode of the clock.
    fn mash_mode(&self) -> GpioResult<MashMode>;
    /// Sets the mash mode of the clock.
    fn set_mash_mode(&mut self, mode: MashMode) -> GpioResult<()>;

    /// Gets the current clock source.
    fn source(&self) -> GpioResult<ClockSource>;
    /// Sets the clock source.
    fn set_source(&mut self, source: ClockSource) -> GpioResult<()>;

    /// Gets the current divisor value.
    fn divisor(&self) -> GpioResult<f32>;
    /// Sets the divisor value.
    fn set_divisor(&mut self, divisor: f32) -> GpioResult<()>;
}

/// Represents the different mash modes available for clock drivers.
/// 
/// Mash modes can be used to control the frequency as a fraction of the source clock frequency,
/// in exchange for lower stability.
/// 
/// DIVI (in the documentation of the specific mash modes) is the floor of the divisor value.
#[derive(Copy, Clone, Debug)]
pub enum MashMode {
    /// No mash mode, normal clock operation. Uses the divisor as an integer.
    None,
    /// Minimum DIVI: 2  
    /// Minimum output frequency: source / (DIVI + 1)  
    /// Maximum output frequency: source / DIVI
    Mash1,
    /// Minimum DIVI: 3  
    /// Minimum output frequency: source / (DIVI + 2)  
    /// Maximum output frequency: source / (DIVI - 1)
    Mash2,
    /// Minimum DIVI: 5  
    /// Minimum output frequency: source / (DIVI + 4)  
    /// Maximum output frequency: source / (DIVI - 3)
    Mash3,
}

impl MashMode {
    /// Converts an index to a [MashMode].
    /// 
    /// Index corresponds to the values in the `CM_CTL` register (0 for none, 1 for Mash1, etc.).
    pub fn from_index(index: u8) -> GpioResult<Self> {
        match index {
            0 => Ok(MashMode::None),
            1 => Ok(MashMode::Mash1),
            2 => Ok(MashMode::Mash2),
            3 => Ok(MashMode::Mash3),
            _ => Err(GpioError::InvalidArgument),
        }
    }
    
    /// Converts a [MashMode] to an index.
    /// 
    /// Index corresponds to the values in the `CM_CTL` register (0 for none, 1 for Mash1, etc.).
    pub fn to_index(&self) -> u8 {
        match self {
            MashMode::None => 0,
            MashMode::Mash1 => 1,
            MashMode::Mash2 => 2,
            MashMode::Mash3 => 3,
        }
    }
}

/// Represents the physical clock source used by the clock driver.
/// 
/// This is mostly undocumented by the official datasheet, that only mentions the names of the sources,
/// but not the specific frequencies. Those have been found in various sources online.
/// 
/// 
#[derive(Copy, Clone, Debug)]
pub enum ClockSource {
    /// Ground, 0 Hz.
    /// 
    /// Presumably will not work.
    Ground,
    /// Oscillator, 19.2 MHz
    /// 
    /// Uses the crystal oscillator on the board.
    Oscillator,
    /// Test/Debug 0, unknown.
    TestDebug0,
    /// Test/Debug 1, unknown.
    TestDebug1,
    /// Phase-locked loop A, varies.
    /// 
    /// Uses the PLL to generate a clock signal.
    PllA,
    /// Phase-locked loop C, 1000 MHz.
    /// 
    /// Uses the PLL to generate a clock signal.
    PllC,
    /// Phase-locked loop D, 500 MHz.
    /// 
    /// Uses the PLL to generate a clock signal.
    PllD,
    /// HDMI Auxiliary, 216 MHz.
    /// 
    /// Uses the HDMI auxiliary clock source.
    HdmiAuxiliary,
}

impl ClockSource {
    /// Converts an index to a [ClockSource].
    /// 
    /// Index corresponds to the values in the `CM_CTL` register (0 for Ground, 1 for Oscillator, etc.).
    /// 
    /// For invalid indices, returns [ClockSource::Ground].
    pub fn from_index(index: u32) -> GpioResult<Self> {
        match index {
            1 => Ok(ClockSource::Oscillator),
            2 => Ok(ClockSource::TestDebug0),
            3 => Ok(ClockSource::TestDebug1),
            4 => Ok(ClockSource::PllA),
            5 => Ok(ClockSource::PllC),
            6 => Ok(ClockSource::PllD),
            7 => Ok(ClockSource::HdmiAuxiliary),
            _ => Ok(ClockSource::Ground),
        }
    }

    /// Converts a [ClockSource] to an index.
    /// 
    /// Index corresponds to the values in the `CM_CTL` register (0 for Ground, 1 for Oscillator, etc.).
    pub fn to_index(&self) -> u32 {
        match self {
            ClockSource::Ground => 0,
            ClockSource::Oscillator => 1,
            ClockSource::TestDebug0 => 2,
            ClockSource::TestDebug1 => 3,
            ClockSource::PllA => 4,
            ClockSource::PllC => 5,
            ClockSource::PllD => 6,
            ClockSource::HdmiAuxiliary => 7,
        }
    }
}
