pub mod raw;

use std::fmt::Debug;
use crate::gpio::{GpioError, GpioResult};

pub trait ClockDriver: Debug {
    fn enabled(&self) -> GpioResult<bool>;
    fn set_enabled(&mut self, enabled: bool) -> GpioResult<()>;

    fn mash_mode(&self) -> GpioResult<MashMode>;
    fn set_mash_mode(&mut self, mode: MashMode) -> GpioResult<()>;

    fn source(&self) -> GpioResult<ClockSource>;
    fn set_source(&mut self, source: ClockSource) -> GpioResult<()>;

    fn divisor(&self) -> GpioResult<f32>;
    fn set_divisor(&mut self, divisor: f32) -> GpioResult<()>;
}

#[derive(Copy, Clone, Debug)]
pub enum MashMode {
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
    pub fn from_index(index: u8) -> GpioResult<Self> {
        match index {
            0 => Ok(MashMode::None),
            1 => Ok(MashMode::Mash1),
            2 => Ok(MashMode::Mash2),
            3 => Ok(MashMode::Mash3),
            _ => Err(GpioError::InvalidArgument),
        }
    }
    
    pub fn to_index(&self) -> u8 {
        match self {
            MashMode::None => 0,
            MashMode::Mash1 => 1,
            MashMode::Mash2 => 2,
            MashMode::Mash3 => 3,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ClockSource {
    /// Unusable
    Ground,
    /// Oscillator, 19.2 MHz
    Oscillator,
    TestDebug0,
    TestDebug1,
    /// PLL A, varies
    PllA,
    /// PLL C, 1000 MHz
    PllC,
    /// PLL D, 500 MHz
    PllD,
    /// HDMI Auxiliary, 216 MHz
    HdmiAuxiliary,
}

impl ClockSource {
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
