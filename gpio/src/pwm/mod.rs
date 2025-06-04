//! PWM functionality.

mod sysfs;
mod ext;
mod raw;

use crate::{GpioError, GpioResult};
use std::fmt::{Debug, Display};
use std::str::FromStr;
pub use sysfs::*;
pub use ext::*;
pub use raw::*;

/// Trait for PWM driver implementations.
/// 
/// For the raw implementation and device-level documentation, see [raw::RawPwmDriver].
/// 
/// PWM (Pulse Width Modulation) lets you set a pin to output specific states periodically.
/// It will be high for a certain duration (duty cycle) and low for the rest of the period.
pub trait PwmDriver: Debug {
    /// Returns the number of PWM pins available in this driver.
    fn count(&self) -> GpioResult<usize>;

    /// Returns a PWM pin by its index.
    fn get_pin(&self, index: usize) -> GpioResult<Box<dyn PwmPin + '_>>;
}

/// Trait for PWM pins, providing methods to control PWM functionality.
pub trait PwmPin: Debug {
    /// Gets the period set for this PWM pin in nanoseconds.
    fn period_ns(&self) -> GpioResult<u32>;
    /// Sets the period for this PWM pin in nanoseconds.
    fn set_period_ns(&mut self, period_ns: u32) -> GpioResult<()>;

    /// Gets the duty cycle set for this PWM pin in nanoseconds.
    fn duty_ns(&self) -> GpioResult<u32>;
    /// Sets the duty cycle for this PWM pin in nanoseconds.
    fn set_duty_ns(&mut self, duty_ns: u32) -> GpioResult<()>;

    /// Gets the polarity of the PWM signal.
    fn polarity(&self) -> GpioResult<PwmPolarity>;
    /// Sets the polarity of the PWM signal.
    fn set_polarity(&mut self, polarity: PwmPolarity) -> GpioResult<()>;

    /// Checks if the PWM channel is enabled.
    fn is_enabled(&self) -> GpioResult<bool>;
    /// Enables the PWM channel, allowing it to output the configured signal.
    fn enable(&mut self) -> GpioResult<()>;
    /// Disables the PWM channel, stopping the output signal.
    fn disable(&mut self) -> GpioResult<()>;
}

/// Represents the polarity of a PWM signal.
#[derive(Copy, Clone, Debug, Default)]
pub enum PwmPolarity {
    /// Normal polarity means the signal is high for the duty cycle duration.
    #[default]
    Normal,
    /// Inversed polarity means the signal is low for the duty cycle duration.
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
