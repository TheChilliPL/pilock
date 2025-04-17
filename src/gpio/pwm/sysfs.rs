use crate::gpio::pwm::{PwmDriver, PwmPin, PwmPolarity};
use crate::gpio::{GpioError, GpioResult};
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct SysfsPwmDriver {
    base_path: PathBuf,
}

impl SysfsPwmDriver {
    pub fn count_chips() -> GpioResult<usize> {
        let path = Path::new("/sys/class/pwm");
        let mut count = 0;
        for index in 0.. {
            let chip_path = path.join(format!("pwmchip{}", index));
            if chip_path.exists() {
                count += 1;
            } else {
                break;
            }
        }
        Ok(count)
    }

    pub fn get_chip(index: usize) -> GpioResult<Self> {
        let path = Path::new("/sys/class/pwm");
        let chip_path = path.join(format!("pwmchip{}", index));
        if !chip_path.exists() {
            return Err(GpioError::InvalidArgument);
        }
        Ok(SysfsPwmDriver { base_path: chip_path })
    }
}

impl Debug for SysfsPwmDriver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SysfsPwmDriver({:?})", self.base_path)
    }
}

impl PwmDriver for SysfsPwmDriver {
    fn count(&self) -> GpioResult<usize> {
        let path = self.base_path.join("npwm");
        let content = std::fs::read_to_string(&path).map_err(|_| GpioError::Other("reading PWM pin count failed".to_string()))?;
        let count: usize = content.trim().parse().map_err(|_| GpioError::Other("parsing PWM pin count failed".to_string()))?;
        Ok(count)
    }

    fn get_pin(&self, index: usize) -> GpioResult<Box<dyn PwmPin>> {
        // Write the pin number to export
        let export_path = self.base_path.join("export");
        std::fs::write(&export_path, index.to_string()).map_err(|_| GpioError::Other("exporting PWM pin failed".to_string()))?;
        let path = self.base_path.join(format!("pwm{}", index));
        if !path.exists() {
            return Err(GpioError::InvalidArgument);
        }
        let pin = SysfsPwmPin { base_path: path };
        Ok(Box::new(pin))
    }
}

pub struct SysfsPwmPin {
    base_path: PathBuf,
}

impl Debug for SysfsPwmPin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SysfsPwmPin({:?})", self.base_path)
    }
}

impl PwmPin for SysfsPwmPin {
    fn period_ns(&self) -> GpioResult<u32> {
        let path = self.base_path.join("period");
        let content = std::fs::read_to_string(&path)?;
        let period: u32 = content.trim().parse().map_err(|_| GpioError::Other("parsing PWM period failed".to_string()))?;
        Ok(period)
    }

    fn set_period_ns(&mut self, period_ns: u32) -> GpioResult<()> {
        let path = self.base_path.join("period");
        std::fs::write(&path, period_ns.to_string())?;
        Ok(())
    }

    fn duty_ns(&self) -> GpioResult<u32> {
        let path = self.base_path.join("duty");
        let content = std::fs::read_to_string(&path)?;
        let duty: u32 = content.trim().parse().map_err(|_| GpioError::Other("parsing PWM duty failed".to_string()))?;
        Ok(duty)
    }

    fn set_duty_ns(&mut self, duty_ns: u32) -> GpioResult<()> {
        let path = self.base_path.join("duty");
        std::fs::write(&path, duty_ns.to_string())?;
        Ok(())
    }

    fn polarity(&self) -> GpioResult<PwmPolarity> {
        let path = self.base_path.join("polarity");
        let content = std::fs::read_to_string(&path)?;
        let polarity = PwmPolarity::from_str(content.trim())?;
        Ok(polarity)
    }

    fn set_polarity(&mut self, polarity: PwmPolarity) -> GpioResult<()> {
        let path = self.base_path.join("polarity");
        std::fs::write(&path, polarity.to_string())?;
        Ok(())
    }

    fn is_enabled(&self) -> GpioResult<bool> {
        let path = self.base_path.join("enable");
        let content = std::fs::read_to_string(&path)?;
        let enabled: bool = match content.trim() {
            "1" => true,
            "0" => false,
            _ => return Err(GpioError::Other("parsing PWM enabled state failed".to_string())),
        };
        Ok(enabled)
    }

    fn enable(&mut self) -> GpioResult<()> {
        let path = self.base_path.join("enable");
        std::fs::write(&path, "1")?;
        Ok(())
    }

    fn disable(&mut self) -> GpioResult<()> {
        let path = self.base_path.join("enable");
        std::fs::write(&path, "0")?;
        Ok(())
    }
}
