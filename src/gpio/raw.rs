use crate::gpio::{GpioBus, GpioBusInput, GpioBusOutput, GpioDriver, GpioError, GpioInput, GpioOutput, GpioPin, GpioResult};
use bitvec::vec::BitVec;
use memmap2::{MmapOptions, MmapRaw};
use std::fmt::{Debug, Formatter};
use std::fs::OpenOptions;
use std::sync::atomic::AtomicU8;

pub struct RawGpioDriver {
    mmap: MmapRaw,
    used_pins: BitVec<AtomicU8>,
}

impl RawGpioDriver {
    #[cfg(target_pointer_width = "64")]
    const GPIO_BASE: u32 = 0xFE200000;
    #[cfg(target_pointer_width = "32")]
    const GPIO_BASE: u32 = 0x3F200000;

    const PIN_COUNT: usize = 58;

    fn create(path: &str) -> GpioResult<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;

        let mmap = MmapOptions::new()
                .offset(Self::GPIO_BASE as u64)
                .len(4096)
                .map_raw(&file)?;

        Ok(RawGpioDriver {
            mmap,
            used_pins: BitVec::repeat(false, Self::PIN_COUNT),
        })
    }
    pub fn new_gpiomem() -> GpioResult<Self> {
        Self::create("/dev/gpiomem")
    }

    pub fn new_mem() -> GpioResult<Self> {
        Self::create("/dev/mem")
    }

    fn get_pin_function(&self, pin_index: usize) -> GpioResult<u32> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_ptr() as *const u32;
        // GPFSELn register
        let register_ptr = unsafe { mmap.add(pin_index / 10) };
        let shift = (pin_index % 10) * 3;

        let register_value = unsafe { register_ptr.read_volatile() };
        let value = (register_value >> shift) & 0b111;
        // trace!("Read pin function: pin_index={} value={}", pin_index, value);
        Ok(value)
    }

    fn set_pin_function(&self, pin_index: usize, function: u8) -> GpioResult<()> {
        if function > 0b111 {
            return Err(GpioError::InvalidArgument);
        }

        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // GPFSELn register
        let register_ptr = unsafe { mmap.add(pin_index / 10) };
        let shift = (pin_index % 10) * 3;

        let mut register_value = unsafe { register_ptr.read_volatile() };
        register_value &= !(0b111 << shift); // Clear the bits for this pin
        register_value |= (function as u32) << shift; // Set the pin to input mode
        unsafe { register_ptr.write_volatile(register_value) };

        // trace!("Set pin function: pin_index={} function={}", pin_index, function);

        Ok(())
    }

    fn set_pin_output(&self, pin_index: usize, high: bool) -> GpioResult<()> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // GPSETn/GPCLRn register
        let register_ptr = unsafe { mmap.add(if high { 0x1c / 4 } else { 0x28 / 4 } + pin_index / 32) };
        let shift = pin_index % 32;

        unsafe { register_ptr.write_volatile(1 << shift) };

        // trace!("Set pin output: pin_index={} high={}", pin_index, high);

        Ok(())
    }

    fn get_pin_level(&self, pin_index: usize) -> GpioResult<bool> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_ptr() as *const u32;
        // GPLEVn register
        let register_ptr = unsafe { mmap.add((0x34 / 4) + pin_index / 32) };
        let shift = pin_index % 32;

        let register_value = unsafe { register_ptr.read_volatile() };
        let level = (register_value >> shift) & 1;
        // trace!("Read pin level: pin_index={} level={}", pin_index, level);
        Ok(level != 0)
    }
}

impl Debug for RawGpioDriver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RawGpioDriver({:?})", self.mmap.as_ptr().addr())
    }
}

impl GpioDriver for RawGpioDriver {
    fn count(&self) -> GpioResult<usize> {
        Ok(Self::PIN_COUNT)
    }

    fn get_pin(&self, index: usize) -> GpioResult<Box<dyn GpioPin + '_>> {
        if index >= self.count()? {
            return Err(GpioError::InvalidArgument);
        }

        if self.used_pins[index] {
            return Err(GpioError::AlreadyInUse);
        }

        self.used_pins.set_aliased(index, true);

        Ok(Box::new(RawGpioPin {
            driver: self,
            pin_index: index,
            // active_level: GpioActiveLevel::High,
            // bias: GpioBias::None,
        }))
    }

    fn get_pin_bus<const N: usize>(&self, indices: [usize; N]) -> GpioResult<Box<dyn GpioBus<N> + '_>> {
        let n = self.count()?;

        if indices.iter().any(|&index| index >= n) {
            return Err(GpioError::InvalidArgument);
        }

        if indices.iter().any(|&index| self.used_pins[index]) {
            return Err(GpioError::AlreadyInUse);
        }

        for &index in &indices {
            self.used_pins.set_aliased(index, true);
        }

        Ok(Box::new(RawGpioBus {
            driver: self,
            pin_indices: indices,
        }))
    }
}

struct RawGpioPin<'a> {
    driver: &'a RawGpioDriver,
    pin_index: usize,
    // active_level: GpioActiveLevel,
    // bias: GpioBias,
}

impl Debug for RawGpioPin<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}]", self.driver, self.pin_index)
    }
}

impl GpioPin for RawGpioPin<'_> {
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioInput + '_>> {
        self.driver.set_pin_function(self.pin_index, 0)?; // Set to input
        Ok(Box::new(RawGpioInput { pin: self }))
    }

    fn as_output(&mut self) -> GpioResult<Box<dyn GpioOutput + '_>> {
        self.driver.set_pin_function(self.pin_index, 1)?; // Set to output
        Ok(Box::new(RawGpioOutput { pin: self }))
    }
}

impl Drop for RawGpioPin<'_> {
    fn drop(&mut self) {
        self.driver.used_pins.set_aliased(self.pin_index, false);
    }
}

struct RawGpioInput<'a> {
    pin: &'a RawGpioPin<'a>,
}

impl Debug for RawGpioInput<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[input]", self.pin)
    }
}

impl GpioInput for RawGpioInput<'_> {
    fn read(&self) -> GpioResult<bool> {
        self.pin.driver.get_pin_level(self.pin.pin_index)
    }
}

struct RawGpioOutput<'a> {
    pin: &'a RawGpioPin<'a>,
}

impl Debug for RawGpioOutput<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[output]", self.pin)
    }
}

impl GpioOutput for RawGpioOutput<'_> {
    fn write(&self, value: bool) -> GpioResult<()> {
        self.pin.driver.set_pin_output(self.pin.pin_index, value)
    }
}

struct RawGpioBus<'a, const N: usize> {
    driver: &'a RawGpioDriver,
    pin_indices: [usize; N],
}

impl<const N: usize> Debug for RawGpioBus<'_, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.driver, self.pin_indices)
    }
}

impl<const N: usize> GpioBus<N> for RawGpioBus<'_, N> {
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioBusInput<N> + '_>> {
        for &pin_index in &self.pin_indices {
            self.driver.set_pin_function(pin_index, 0)?; // Set to input
        }
        Ok(Box::new(RawGpioBusInput { bus: self }))
    }

    fn as_output(&mut self) -> GpioResult<Box<dyn GpioBusOutput<N> + '_>> {
        for &pin_index in &self.pin_indices {
            self.driver.set_pin_function(pin_index, 1)?; // Set to output
        }
        Ok(Box::new(RawGpioBusOutput { bus: self }))
    }
}

impl<const N: usize> Drop for RawGpioBus<'_, N> {
    fn drop(&mut self) {
        for &pin_index in &self.pin_indices {
            for &pin_index in &self.pin_indices {
                _ = self.driver.set_pin_function(pin_index, 0); // Set to input
            }
            self.driver.used_pins.set_aliased(pin_index, false);
        }
    }
}

struct RawGpioBusInput<'a, const N: usize> {
    bus: &'a RawGpioBus<'a, N>,
}

impl<const N: usize> Debug for RawGpioBusInput<'_, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[input]", self.bus)
    }
}

impl<const N: usize> GpioBusInput<N> for RawGpioBusInput<'_, N> {
    fn read(&self) -> GpioResult<[bool; N]> {
        let mut values = [false; N];
        for (i, &pin_index) in self.bus.pin_indices.iter().enumerate() {
            values[i] = self.bus.driver.get_pin_level(pin_index)?;
        }
        Ok(values)
    }
}

struct RawGpioBusOutput<'a, const N: usize> {
    bus: &'a RawGpioBus<'a, N>,
}

impl<const N: usize> Debug for RawGpioBusOutput<'_, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[output]", self.bus)
    }
}

impl<const N: usize> GpioBusOutput<N> for RawGpioBusOutput<'_, N> {
    fn write(&self, values: &[bool; N]) -> GpioResult<()> {
        for (i, &pin_index) in self.bus.pin_indices.iter().enumerate() {
            self.bus.driver.set_pin_output(pin_index, values[i])?;
        }
        Ok(())
    }
}
