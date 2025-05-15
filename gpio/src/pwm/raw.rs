use std::fmt::Debug;
use std::fs::OpenOptions;
use std::sync::atomic::AtomicU8;
use bitvec::vec::BitVec;
use log::debug;
use memmap2::{MmapOptions, MmapRaw};
use crate::{GpioError, GpioResult};
use crate::pwm::{PwmDriver, PwmPin, PwmPolarity};

/// Raw PWM driver for the Raspberry Pi.
/// 
/// Requires `/dev/mem` access, so root privileges are needed.
/// 
/// Requires the pins to be set to the correct function before using this driver manually.
/// Assumes the clock is set to 20 MHz, which is below the maximum supported frequency for PWM. It can
/// be done with [ClockDriver](crate::clock::ClockDriver) set to [ClockSource::PllC](crate::clock::ClockSource::PllC)
/// with a divisor of `50.0`.
pub struct RawPwmDriver {
    mmap: MmapRaw,
    chip_index: u8,
    used_pins: BitVec<AtomicU8>,
}

impl RawPwmDriver {
    // #[cfg(target_pointer_width = "64")]
    // const PWM_BASE: u32 = 0xFE20C000;
    // #[cfg(target_pointer_width = "32")]
    const PWM_BASE: u32 = 0x3F20C000;

    const CHIP_COUNT: usize = 2;
    const PIN_COUNT: usize = 2;

    fn create(path: &str, chip_index: usize) -> GpioResult<Self> {
        if chip_index >= Self::CHIP_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;

        let mmap = MmapOptions::new()
            .offset(Self::PWM_BASE as u64 + chip_index as u64 * 0x800)
            // .len(4096)
            .len(0x28)
            .map_raw(&file)?;

        Ok(RawPwmDriver {
            mmap,
            chip_index: chip_index as u8,
            used_pins: BitVec::repeat(false, Self::PIN_COUNT),
        })
    }

    pub fn new_mem(chip_index: usize) -> GpioResult<Self> {
        Self::create("/dev/mem", chip_index)
    }

    fn init_channel(&self, pin_index: usize) -> GpioResult<()> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // PWM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };
        let shift = pin_index * 8;

        let mask: u32 = ((1 << 8) - 1) << shift;

        let mut value: u32 = 0;
        value |= 1; // Channel enable (PWEN)
        // value |= 0 << 1; // PWM mode (MODE)
        value |= 1 << 7; // M/S transmission mode (MSEN)

        let mut register_value = unsafe { register_ptr.read_volatile() };
        register_value &= !mask; // Clear the bits
        register_value |= value << shift; // Set the new value
        unsafe { register_ptr.write_volatile(register_value) };

        Ok(())
    }

    fn reset_channel(&self, pin_index: usize) -> GpioResult<()> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // PWM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };
        let shift = pin_index * 8;

        let mask: u32 = ((1 << 8) - 1) << shift;

        let mut register_value = unsafe { register_ptr.read_volatile() };
        register_value &= !mask; // Clear the bits
        unsafe { register_ptr.write_volatile(register_value) };

        Ok(())
    }

    fn get_period(&self, pin_index: usize) -> GpioResult<u32> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_ptr() as *const u32;
        // PWM_RNGi register
        let register_ptr = unsafe { mmap.add(0x10 / 4 + 0x10 / 4 * pin_index) };

        let value = unsafe { register_ptr.read_volatile() };
        Ok(value)
    }

    fn set_period(&self, pin_index: usize, period: u32) -> GpioResult<()> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // PWM_RNGi register
        let register_ptr = unsafe { mmap.add(0x10 / 4 + 0x10 / 4 * pin_index) };

        let mut register_value = unsafe { register_ptr.read_volatile() };
        register_value = period; // Set the new value
        unsafe { register_ptr.write_volatile(register_value) };
        
        debug!("Set PWM period: pin_index={} period={}", pin_index, period);

        Ok(())
    }

    fn get_duty(&self, pin_index: usize) -> GpioResult<u32> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_ptr() as *const u32;
        // PWM_DATi register
        let register_ptr = unsafe { mmap.add(0x14 / 4 + 0x10 / 4 * pin_index) };

        let value = unsafe { register_ptr.read_volatile() };
        Ok(value)
    }

    fn set_duty(&self, pin_index: usize, duty: u32) -> GpioResult<()> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // PWM_DATi register
        let register_ptr = unsafe { mmap.add(0x14 / 4 + 0x10 / 4 * pin_index) };

        let mut register_value = unsafe { register_ptr.read_volatile() };
        register_value = duty; // Set the new value
        unsafe { register_ptr.write_volatile(register_value) };
        
        debug!("Set PWM duty: pin_index={} duty={}", pin_index, duty);

        Ok(())
    }
}

impl Debug for RawPwmDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RawPwmDriver({:?}, {})", self.mmap.as_ptr().addr(), self.chip_index)
    }
}

impl PwmDriver for RawPwmDriver {
    fn count(&self) -> GpioResult<usize> {
        Ok(Self::PIN_COUNT)
    }

    fn get_pin(&self, index: usize) -> GpioResult<Box<dyn PwmPin + '_>> {
        if index >= self.count()? {
            return Err(GpioError::InvalidArgument);
        }

        if self.used_pins[index] {
            return Err(GpioError::AlreadyInUse);
        }

        self.used_pins.set_aliased(index, true);

        debug!("Using PWM pin {} on chip {}", index, self.chip_index);

        Ok(Box::new(RawPwmPin {
            driver: self,
            pin_index: index,
        }))
    }
}

pub struct RawPwmPin<'a> {
    driver: &'a RawPwmDriver,
    pin_index: usize,
}

impl Debug for RawPwmPin<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}]", self.driver, self.pin_index)
    }
}

impl PwmPin for RawPwmPin<'_> {
    fn period_ns(&self) -> GpioResult<u32> {
        self.driver.get_period(self.pin_index).map(|v| v * 50)
    }

    fn set_period_ns(&mut self, period_ns: u32) -> GpioResult<()> {
        let cycles = period_ns / 50;
        self.driver.set_period(self.pin_index, cycles)
    }

    fn duty_ns(&self) -> GpioResult<u32> {
        self.driver.get_duty(self.pin_index).map(|v| v * 50)
    }

    fn set_duty_ns(&mut self, duty_ns: u32) -> GpioResult<()> {
        let cycles = duty_ns / 50;
        self.driver.set_duty(self.pin_index, cycles)
    }

    fn polarity(&self) -> GpioResult<PwmPolarity> {
        Ok(PwmPolarity::Normal)
    }

    fn set_polarity(&mut self, polarity: PwmPolarity) -> GpioResult<()> {
        todo!()
    }

    fn is_enabled(&self) -> GpioResult<bool> {
        todo!()
    }

    fn enable(&mut self) -> GpioResult<()> {
        self.driver.init_channel(self.pin_index)
    }

    fn disable(&mut self) -> GpioResult<()> {
        self.driver.reset_channel(self.pin_index)
    }
}
