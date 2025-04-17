use std::time::Duration;
use crate::gpio::GpioResult;
use crate::gpio::pwm::PwmPin;

pub trait PwmExtension {
    fn period(&self) -> GpioResult<Duration>;
    fn set_period(&mut self, period: Duration) -> GpioResult<()>;

    fn duty(&self) -> GpioResult<Duration>;
    fn set_duty(&mut self, duty: Duration) -> GpioResult<()>;
}

impl PwmExtension for dyn PwmPin {
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
