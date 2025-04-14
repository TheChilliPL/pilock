use crate::gpio::{
    GpioBus, GpioBusInput, GpioBusOutput, GpioDriver, GpioError, GpioInput, GpioOutput, GpioPin,
    GpioResult,
};
use bitvec::vec::BitVec;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::AtomicU8;

pub struct GpiodDriver {
    chip: gpiod::Chip,
    used_pins: BitVec<AtomicU8>,
}

impl GpiodDriver {
    pub fn new(chip: gpiod::Chip) -> Self {
        let n = chip.num_lines() as usize;
        let bits = BitVec::repeat(false, n);
        Self {
            chip,
            used_pins: bits,
        }
    }
}

impl Debug for GpiodDriver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GpiodDriver({})", self.chip.name())
    }
}

impl GpioDriver for GpiodDriver {
    fn count(&self) -> GpioResult<usize> {
        Ok(self.chip.num_lines() as usize)
    }

    fn get_pin(&self, index: usize) -> GpioResult<Box<dyn GpioPin + '_>> {
        if index >= self.count()? {
            return Err(GpioError::InvalidArgument);
        }

        if self.used_pins[index] {
            return Err(GpioError::AlreadyInUse);
        }

        self.used_pins.set_aliased(index, true);

        Ok(Box::new(GpiodPin {
            driver: self,
            pin_index: index,
        }))
    }

    fn get_pin_bus<const N: usize>(
        &self,
        indices: [usize; N],
    ) -> GpioResult<Box<dyn GpioBus<N> + '_>> {
        let n = self.count()?;

        if indices.iter().any(|&index| index >= n) {
            return Err(GpioError::InvalidArgument);
        }

        if indices.iter().any(|&index| self.used_pins[index]) {
            return Err(GpioError::AlreadyInUse);
        }

        for index in indices {
            self.used_pins.set_aliased(index, true);
        }

        Ok(Box::new(GpiodBus {
            driver: self,
            pin_indices: indices,
        }))
    }
}

struct GpiodPin<'a> {
    driver: &'a GpiodDriver,
    pin_index: usize,
}

impl Debug for GpiodPin<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}]", self.driver, self.pin_index)
    }
}

impl GpioPin for GpiodPin<'_> {
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioInput + '_>> {
        let line = self
            .driver
            .chip
            .request_lines(gpiod::Options::input([self.pin_index as u32]))?;
        let input = GpiodInput { pin: self, line };
        Ok(Box::new(input))
    }

    fn as_output(&mut self) -> GpioResult<Box<dyn GpioOutput + '_>> {
        let line = self
            .driver
            .chip
            .request_lines(gpiod::Options::output([self.pin_index as u32]))?;
        let output = GpiodOutput { pin: self, line };
        Ok(Box::new(output))
    }
}

impl Drop for GpiodPin<'_> {
    fn drop(&mut self) {
        self.driver.used_pins.set_aliased(self.pin_index, false);
    }
}

struct GpiodInput<'a> {
    pin: &'a GpiodPin<'a>,
    line: gpiod::Lines<gpiod::Input>,
}

impl Debug for GpiodInput<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}][input]", self.pin.driver, self.pin.pin_index)
    }
}

impl GpioInput for GpiodInput<'_> {
    fn read(&self) -> GpioResult<bool> {
        let values = self.line.get_values([false])?;
        Ok(values[0])
    }
}

struct GpiodOutput<'a> {
    pin: &'a GpiodPin<'a>,
    line: gpiod::Lines<gpiod::Output>,
}

impl Debug for GpiodOutput<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}][output]", self.pin.driver, self.pin.pin_index)
    }
}

impl GpioOutput for GpiodOutput<'_> {
    fn write(&self, value: bool) -> GpioResult<()> {
        self.line.set_values([value])?;
        Ok(())
    }
}

struct GpiodBus<'a, const N: usize> {
    driver: &'a GpiodDriver,
    pin_indices: [usize; N],
}

impl<const N: usize> Debug for GpiodBus<'_, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.driver, self.pin_indices)
    }
}

impl<const N: usize> GpioBus<N> for GpiodBus<'_, N> {
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioBusInput<N> + '_>> {
        let line = self.driver.chip.request_lines(gpiod::Options::input(
            self.pin_indices
                .iter()
                .map(|&index| index as u32)
                .collect::<Vec<_>>(),
        ))?;
        let input = GpiodBusInput { bus: self, line };
        Ok(Box::new(input))
    }

    fn as_output(&mut self) -> GpioResult<Box<dyn GpioBusOutput<N> + '_>> {
        let line = self.driver.chip.request_lines(gpiod::Options::output(
            self.pin_indices
                .iter()
                .map(|&index| index as u32)
                .collect::<Vec<_>>(),
        ))?;
        let output = GpiodBusOutput { bus: self, line };
        Ok(Box::new(output))
    }
}

impl<const N: usize> Drop for GpiodBus<'_, N> {
    fn drop(&mut self) {
        for &index in &self.pin_indices {
            self.driver.used_pins.set_aliased(index, false);
        }
    }
}

struct GpiodBusInput<'a, const N: usize> {
    bus: &'a GpiodBus<'a, N>,
    line: gpiod::Lines<gpiod::Input>,
}

impl<const N: usize> Debug for GpiodBusInput<'_, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}[input]", self.bus.driver, self.bus.pin_indices)
    }
}

impl<const N: usize> GpioBusInput<N> for GpiodBusInput<'_, N> {
    fn read(&self) -> GpioResult<[bool; N]> {
        let values = self.line.get_values([false; N])?;
        Ok(values)
    }
}

struct GpiodBusOutput<'a, const N: usize> {
    bus: &'a GpiodBus<'a, N>,
    line: gpiod::Lines<gpiod::Output>,
}

impl<const N: usize> Debug for GpiodBusOutput<'_, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}[output]", self.bus.driver, self.bus.pin_indices)
    }
}

impl<const N: usize> GpioBusOutput<N> for GpiodBusOutput<'_, N> {
    fn write(&self, values: &[bool; N]) -> GpioResult<()> {
        let mut gpiod_values = [false; N];
        for (i, &value) in values.iter().enumerate() {
            gpiod_values[i] = value;
        }
        self.line.set_values(gpiod_values)?;
        Ok(())
    }
}
