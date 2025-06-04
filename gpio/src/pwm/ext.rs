//! Extension traits for PWM pins.

use std::time::Duration;
use crate::GpioResult;
use crate::pwm::PwmPin;

/// Extension trait for PWM pins, providing methods to work with durations instead of raw nanoseconds.
pub trait PwmExtension {
    /// Gets the period of the PWM pin as a [Duration].
    fn period(&self) -> GpioResult<Duration>;
    /// Sets the period of the PWM pin using a [Duration].
    fn set_period(&mut self, period: Duration) -> GpioResult<()>;

    /// Gets the duty cycle of the PWM pin as a [Duration].
    fn duty(&self) -> GpioResult<Duration>;
    /// Sets the duty cycle of the PWM pin using a [Duration].
    fn set_duty(&mut self, duty: Duration) -> GpioResult<()>;
}

impl PwmExtension for dyn PwmPin + '_ {
    fn period(&self) -> GpioResult<Duration> {
        let period_ns = self.period_ns()?;
        Ok(Duration::from_nanos(period_ns.into()))
    }

    fn set_period(&mut self, period: Duration) -> GpioResult<()> {
        let period_ns = period.as_nanos() as u32;
        self.set_period_ns(period_ns)
    }

    fn duty(&self) -> GpioResult<Duration> {
        let duty_ns = self.duty_ns()?;
        Ok(Duration::from_nanos(duty_ns.into()))
    }

    fn set_duty(&mut self, duty: Duration) -> GpioResult<()> {
        let duty_ns = duty.as_nanos() as u32;
        self.set_duty_ns(duty_ns)
    }
}
