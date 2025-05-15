mod sysfs;
mod ext;
mod raw;

use crate::{GpioError, GpioResult};
use std::fmt::{Debug, Display};
use std::str::FromStr;
pub use sysfs::*;
pub use ext::*;
pub use raw::*;

pub trait PwmDriver: Debug {
    fn count(&self) -> GpioResult<usize>;

    fn get_pin(&self, index: usize) -> GpioResult<Box<dyn PwmPin + '_>>;
}

pub trait PwmPin: Debug {
    fn period_ns(&self) -> GpioResult<u32>;
    fn set_period_ns(&mut self, period_ns: u32) -> GpioResult<()>;

    fn duty_ns(&self) -> GpioResult<u32>;
    fn set_duty_ns(&mut self, duty_ns: u32) -> GpioResult<()>;

    fn polarity(&self) -> GpioResult<PwmPolarity>;
    fn set_polarity(&mut self, polarity: PwmPolarity) -> GpioResult<()>;

    fn is_enabled(&self) -> GpioResult<bool>;
    fn enable(&mut self) -> GpioResult<()>;
    fn disable(&mut self) -> GpioResult<()>;
}

#[derive(Copy, Clone, Debug, Default)]
pub enum PwmPolarity {
    #[default]
    Normal,
    Inversed,
}

impl FromStr for PwmPolarity {
    type Err = GpioError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(PwmPolarity::Normal),
            "inversed" => Ok(PwmPolarity::Inversed),
            _ => Err(GpioError::Other("parsing PWM polarity failed".to_string())),
        }
    }
}

impl Display for PwmPolarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            PwmPolarity::Normal => "normal",
            PwmPolarity::Inversed => "inversed",
        };
        write!(f, "{}", str)
    }
}
