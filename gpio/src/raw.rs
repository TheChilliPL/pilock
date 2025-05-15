use crate::{GpioActiveLevel, GpioBias, GpioBus, GpioBusInput, GpioBusOutput, GpioDriveMode, GpioDriver, GpioError, GpioInput, GpioOutput, GpioPin, GpioResult};
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
    // 0x7e200000
    // #[cfg(target_pointer_width = "64")]
    // const GPIO_BASE: u32 = 0xFE200000;
    // #[cfg(target_pointer_width = "32")]
    const GPIO_BASE: u32 = 0x3F200000;
    // const GPIO_BASE: u32 = 0x7E200000;
    // const GPIO_BASE: u32 = 0x20200000;

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

    pub fn raw_get_pin_function(&self, pin_index: usize) -> GpioResult<u32> {
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

    pub fn raw_set_pin_function(&self, pin_index: usize, function: u8) -> GpioResult<()> {
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

    pub(crate) fn raw_set_pin_output(&self, pin_index: usize, high: bool) -> GpioResult<()> {
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

    pub(crate) fn raw_get_pin_level(&self, pin_index: usize) -> GpioResult<bool> {
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

    pub(crate) fn drive_pin(&self, pin_index: usize, high: bool, mode: GpioDriveMode) -> GpioResult<()> {
        let output = mode.get_state(high);

        match output {
            Some(output) => {
                self.raw_set_pin_function(pin_index, 1)?; // Set to output
                self.raw_set_pin_output(pin_index, output)?; // Set output level
            }
            None => {
                self.raw_set_pin_function(pin_index, 0)?; // Set to input
            }
        }

        Ok(())
    }

    pub(crate) fn raw_set_bias(&self, pin_index: usize, bias: GpioBias) -> GpioResult<()> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let bias_value = match bias {
            GpioBias::None => 0b00,
            GpioBias::PullUp => 0b01,
            GpioBias::PullDown => 0b10,
        };

        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // GPIO_PUP_PDN_CNTRL_REGn register (yes that is a long name)
        let register_ptr = unsafe { mmap.add(0xE4 / 4 + pin_index / 16) };
        let shift = (pin_index % 16) * 2;
        let mut register_value = unsafe { register_ptr.read_volatile() };
        register_value &= !(0b11 << shift); // Clear the bits for this pin
        register_value |= bias_value << shift; // Set the bias

        unsafe { register_ptr.write_volatile(register_value) };

        Ok(())
    }

    pub(crate) fn raw_get_bias(&self, pin_index: usize) -> GpioResult<GpioBias> {
        if pin_index >= Self::PIN_COUNT {
            return Err(GpioError::InvalidArgument);
        }

        let mmap = self.mmap.as_ptr() as *const u32;
        // GPIO_PUP_PDN_CNTRL_REGn register (yes that is a long name)
        let register_ptr = unsafe { mmap.add(0xE4 / 4 + pin_index / 16) };
        let shift = (pin_index % 16) * 2;
        let register_value = unsafe { register_ptr.read_volatile() };
        let bias_value = (register_value >> shift) & 0b11;

        let bias = match bias_value {
            0b00 => GpioBias::None,
            0b01 => GpioBias::PullUp,
            0b10 => GpioBias::PullDown,
            _ => return Err(GpioError::NotSupported),
        };
        Ok(bias)
    }

    pub(crate) fn raw_reset(&self, pin_index: usize) -> GpioResult<()> {
        self.raw_set_pin_function(pin_index, 0)?;
        self.raw_set_bias(pin_index, GpioBias::None)?;
        self.raw_set_pin_output(pin_index, false)?;
        Ok(())
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
        self.raw_reset(index)?;

        Ok(Box::new(RawGpioPin {
            driver: self,
            pin_index: index,
            active_level: GpioActiveLevel::High,
            // bias: GpioBias::None,
            drive_mode: GpioDriveMode::PushPull,
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
            self.raw_reset(index)?;
        }

        Ok(Box::new(RawGpioBus {
            driver: self,
            pin_indices: indices,
            active_level: GpioActiveLevel::High,
            drive_mode: GpioDriveMode::PushPull,
        }))
    }
}

struct RawGpioPin<'a> {
    driver: &'a RawGpioDriver,
    pin_index: usize,
    active_level: GpioActiveLevel,
    // bias: GpioBias,
    drive_mode: GpioDriveMode,
}

impl Debug for RawGpioPin<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}]", self.driver, self.pin_index)
    }
}

impl GpioPin for RawGpioPin<'_> {
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioInput + '_>> {
        self.driver.raw_set_pin_function(self.pin_index, 0)?; // Set to input
        Ok(Box::new(RawGpioInput { pin: self }))
    }

    fn as_output(&mut self) -> GpioResult<Box<dyn GpioOutput + '_>> {
        self.driver.raw_set_pin_function(self.pin_index, 1)?; // Set to output
        Ok(Box::new(RawGpioOutput { pin: self }))
    }

    fn supports_active_level(&self) -> bool {
        true
    }

    fn active_level(&self) -> GpioActiveLevel {
        self.active_level
    }

    fn set_active_level(&mut self, level: GpioActiveLevel) -> GpioResult<()> {
        self.active_level = level;
        Ok(())
    }

    fn supports_bias(&self) -> bool {
        true
    }

    fn bias(&self) -> GpioBias {
        self.driver.raw_get_bias(self.pin_index).unwrap_or(GpioBias::None)
    }

    fn set_bias(&mut self, bias: GpioBias) -> GpioResult<()> {
        self.driver.raw_set_bias(self.pin_index, bias)?;
        Ok(())
    }

    fn supports_drive_mode(&self) -> bool {
        true
    }

    fn drive_mode(&self) -> GpioDriveMode {
        self.drive_mode
    }

    fn set_drive_mode(&mut self, mode: GpioDriveMode) -> GpioResult<()> {
        self.drive_mode = mode;
        Ok(())
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
        Ok(self.pin.active_level.get_state(self.pin.driver.raw_get_pin_level(self.pin.pin_index)?))
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
        // self.pin.driver.raw_set_pin_output(self.pin.pin_index, value)
        self.pin.driver.drive_pin(self.pin.pin_index, self.pin.active_level.get_state(value), self.pin.drive_mode)
    }
}

struct RawGpioBus<'a, const N: usize> {
    driver: &'a RawGpioDriver,
    pin_indices: [usize; N],
    active_level: GpioActiveLevel,
    drive_mode: GpioDriveMode,

}

impl<const N: usize> Debug for RawGpioBus<'_, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.driver, self.pin_indices)
    }
}

impl<const N: usize> GpioBus<N> for RawGpioBus<'_, N> {
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioBusInput<N> + '_>> {
        for &pin_index in &self.pin_indices {
            self.driver.raw_set_pin_function(pin_index, 0)?; // Set to input
        }
        Ok(Box::new(RawGpioBusInput { bus: self }))
    }

    fn as_output(&mut self) -> GpioResult<Box<dyn GpioBusOutput<N> + '_>> {
        for &pin_index in &self.pin_indices {
            self.driver.raw_set_pin_function(pin_index, 1)?; // Set to output
        }
        Ok(Box::new(RawGpioBusOutput { bus: self }))
    }

    fn supports_active_level(&self) -> bool {
        true
    }

    fn active_level(&self) -> GpioActiveLevel {
        self.active_level
    }

    fn set_active_level(&mut self, level: GpioActiveLevel) -> GpioResult<()> {
        self.active_level = level;
        Ok(())
    }

    fn supports_bias(&self) -> bool {
        true
    }

    fn bias(&self) -> GpioBias {
        self.driver.raw_get_bias(self.pin_indices[0]).unwrap_or(GpioBias::None)
    }

    fn set_bias(&mut self, bias: GpioBias) -> GpioResult<()> {
        for &pin_index in &self.pin_indices {
            self.driver.raw_set_bias(pin_index, bias)?;
        }
        Ok(())
    }

    fn supports_drive_mode(&self) -> bool {
        true
    }

    fn drive_mode(&self) -> GpioDriveMode {
        self.drive_mode
    }

    fn set_drive_mode(&mut self, mode: GpioDriveMode) -> GpioResult<()> {
        self.drive_mode = mode;
        for &pin_index in &self.pin_indices {
            self.driver.drive_pin(pin_index, false, mode)?;
        }
        Ok(())
    }
}

impl<const N: usize> Drop for RawGpioBus<'_, N> {
    fn drop(&mut self) {
        for &pin_index in &self.pin_indices {
            for &pin_index in &self.pin_indices {
                _ = self.driver.raw_set_pin_function(pin_index, 0); // Set to input
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
            values[i] = self.bus.active_level.get_state(self.bus.driver.raw_get_pin_level(pin_index)?);
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
            // self.bus.driver.raw_set_pin_output(pin_index, values[i])?;
            self.bus.driver.drive_pin(pin_index, self.bus.active_level.get_state(values[i]), self.bus.drive_mode)?;
        }
        Ok(())
    }
}
