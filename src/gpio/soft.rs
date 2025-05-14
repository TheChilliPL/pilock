use crate::gpio::{GpioActiveLevel, GpioBias, GpioBus, GpioBusInput, GpioBusOutput, GpioDriveMode, GpioError, GpioInput, GpioOutput, GpioPin, GpioResult};
use std::fmt::Debug;

pub struct SoftGpioBus<'a, const N: usize> {
    pins: [&'a mut dyn GpioPin; N],
}

impl <'a, const N: usize> SoftGpioBus<'a, N> {
    pub fn new(pins: [&'a mut dyn GpioPin; N]) -> Self {
        Self { pins }
    }
}

impl <const N: usize> Debug for SoftGpioBus<'_, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SoftGpioBus({:?})", self.pins)
    }
}

impl <const N: usize> GpioBus<N> for SoftGpioBus<'_, N> {
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioBusInput<N> + '_>> {
        todo!()
    }

    fn as_output(&mut self) -> GpioResult<Box<dyn GpioBusOutput<N> + '_>> {
        todo!()
    }

    fn supports_active_level(&self) -> bool {
        self.pins.iter().all(|pin| pin.supports_active_level())
    }

    fn active_level(&self) -> GpioActiveLevel {
        self.pins[0].active_level()
    }

    fn set_active_level(&mut self, level: GpioActiveLevel) -> GpioResult<()> {
        if !self.supports_active_level() {
            return Err(GpioError::NotSupported);
        }

        for pin in self.pins.iter_mut() {
            pin.set_active_level(level)?;
        }
        Ok(())
    }

    fn supports_bias(&self) -> bool {
        self.pins.iter().all(|pin| pin.supports_bias())
    }

    fn bias(&self) -> GpioBias {
        self.pins[0].bias()
    }

    fn set_bias(&mut self, bias: GpioBias) -> GpioResult<()> {
        if !self.supports_bias() {
            return Err(GpioError::NotSupported);
        }

        for pin in self.pins.iter_mut() {
            pin.set_bias(bias)?;
        }
        Ok(())
    }

    fn supports_drive_mode(&self) -> bool {
        self.pins.iter().all(|pin| pin.supports_drive_mode())
    }

    fn drive_mode(&self) -> GpioDriveMode {
        self.pins[0].drive_mode()
    }

    fn set_drive_mode(&mut self, mode: GpioDriveMode) -> GpioResult<()> {
        if !self.supports_drive_mode() {
            return Err(GpioError::NotSupported);
        }

        for pin in self.pins.iter_mut() {
            pin.set_drive_mode(mode)?;
        }
        Ok(())
    }
}

pub struct SoftGpioBusInput<'a, const N: usize> {
    pins: [&'a mut dyn GpioInput; N],
}

impl <'a, const N: usize> SoftGpioBusInput<'a, N> {
    pub fn new(pins: [&'a mut dyn GpioInput; N]) -> Self {
        Self { pins }
    }
}

impl <const N: usize> Debug for SoftGpioBusInput<'_, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SoftGpioBusInput({:?})", self.pins)
    }
}

impl <const N: usize> GpioBusInput<N> for SoftGpioBusInput<'_, N> {
    fn read(&self) -> GpioResult<[bool; N]> {
        let mut values = [false; N];
        for (i, pin) in self.pins.iter().enumerate() {
            values[i] = pin.read()?;
        }
        Ok(values)
    }
}

pub struct SoftGpioBusOutput<'a, const N: usize> {
    pins: [&'a mut dyn GpioOutput; N],
}

impl <'a, const N: usize> SoftGpioBusOutput<'a, N> {
    pub fn new(pins: [&'a mut dyn GpioOutput; N]) -> Self {
        Self { pins }
    }
}

impl <const N: usize> Debug for SoftGpioBusOutput<'_, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SoftGpioBusOutput({:?})", self.pins)
    }
}

impl <const N: usize> GpioBusOutput<N> for SoftGpioBusOutput<'_, N> {
    fn write(&self, values: &[bool; N]) -> GpioResult<()> {
        for (i, pin) in self.pins.iter().enumerate() {
            pin.write(values[i])?;
        }
        
        Ok(())
    }
}
