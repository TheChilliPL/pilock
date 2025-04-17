pub mod gpiod;
pub mod lcd;
pub mod debounce;
pub mod pwm;

use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq, Clone)]
pub enum GpioError {
    #[error("pin already in use")]
    AlreadyInUse,
    #[error("invalid argument")]
    InvalidArgument,
    #[error("the feature is not supported on this backend")]
    NotSupported,
    #[error("IO error: {0}")]
    Io(std::io::ErrorKind),
    #[error("error: {0}")]
    Other(String),
}

impl From<std::io::Error> for GpioError {
    fn from(err: std::io::Error) -> Self {
        GpioError::Io(err.kind())
    }
}

pub type GpioResult<T> = Result<T, GpioError>;

pub trait GpioDriver: Debug {
    /// Gets the amount of GPIO pins available.
    fn count(&self) -> GpioResult<usize>;

    /// Gets the GPIO pin at the given index.
    fn get_pin(&self, index: usize) -> GpioResult<Box<dyn GpioPin + '_>>;

    /// Gets the GPIO pin bus at the specific indices.
    fn get_pin_bus<const N: usize>(
        &self,
        indices: [usize; N],
    ) -> GpioResult<Box<dyn GpioBus<N> + '_>>;
}

#[derive(Copy, Clone, Debug)]
pub enum GpioActiveLevel {
    High,
    Low,
}

#[derive(Copy, Clone, Debug)]
pub enum GpioBias {
    None,
    PullUp,
    PullDown,
}

pub trait GpioPin: Debug {
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioInput + '_>>;
    fn as_output(&mut self) -> GpioResult<Box<dyn GpioOutput + '_>>;

    fn supports_active_level(&self) -> bool {
        false
    }
    fn active_level(&self) -> GpioActiveLevel {
        GpioActiveLevel::High
    }
    fn set_active_level(&mut self, _level: GpioActiveLevel) -> GpioResult<()> {
        Err(GpioError::NotSupported)
    }
    fn with_active_level(mut self, level: GpioActiveLevel) -> GpioResult<Self>
    where
        Self: Sized,
    {
        self.set_active_level(level)?;
        Ok(self)
    }

    fn supports_bias(&self) -> bool {
        false
    }
    fn bias(&self) -> GpioBias {
        GpioBias::None
    }
    fn set_bias(&mut self, _bias: GpioBias) -> GpioResult<()> {
        Err(GpioError::NotSupported)
    }
    fn with_bias(mut self, bias: GpioBias) -> GpioResult<Self>
    where
        Self: Sized,
    {
        self.set_bias(bias)?;
        Ok(self)
    }
}

pub trait GpioInput: Debug {
    fn read(&self) -> GpioResult<bool>;
}

pub trait GpioOutput: Debug {
    fn write(&self, value: bool) -> GpioResult<()>;
}

pub trait GpioBus<const N: usize>: Debug {
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioBusInput<N> + '_>>;
    fn as_output(&mut self) -> GpioResult<Box<dyn GpioBusOutput<N> + '_>>;

    fn supports_active_level(&self) -> bool {
        false
    }
    fn active_level(&self) -> GpioActiveLevel {
        GpioActiveLevel::High
    }
    fn set_active_level(&mut self, _level: GpioActiveLevel) -> GpioResult<()> {
        Err(GpioError::NotSupported)
    }
    fn with_active_level(mut self, level: GpioActiveLevel) -> GpioResult<Self>
    where
        Self: Sized,
    {
        self.set_active_level(level)?;
        Ok(self)
    }

    fn supports_bias(&self) -> bool {
        false
    }
    fn bias(&self) -> GpioBias {
        GpioBias::None
    }
    fn set_bias(&mut self, _bias: GpioBias) -> GpioResult<()> {
        Err(GpioError::NotSupported)
    }
    fn with_bias(mut self, bias: GpioBias) -> GpioResult<Self>
    where
        Self: Sized,
    {
        self.set_bias(bias)?;
        Ok(self)
    }
}

pub trait GpioBusInput<const N: usize>: Debug {
    fn read(&self) -> GpioResult<[bool; N]>;
}

impl dyn GpioBusInput<8> + '_ {
    /// Reads the values of the GPIO pins in the bus.
    /// Returns them as a byte, LSb first.
    pub fn read_byte(&self) -> GpioResult<u8> {
        let values = self.read()?;
        let mut byte = 0u8;
        for i in 0..8 {
            if values[i] {
                byte |= 1 << i;
            }
        }
        Ok(byte)
    }
}

impl dyn GpioBusInput<4> + '_ {
    /// Reads the values of the GPIO pins in the bus.
    /// Returns them as a nibble, LSb first.
    pub fn read_nibble(&self) -> GpioResult<u8> {
        let values = self.read()?;
        let mut nibble = 0u8;
        for i in 0..4 {
            if values[i] {
                nibble |= 1 << i;
            }
        }
        Ok(nibble)
    }
}

pub trait GpioBusOutput<const N: usize>: Debug {
    fn write(&self, values: &[bool; N]) -> GpioResult<()>;
}

impl dyn GpioBusOutput<8> + '_ {
    /// Writes the values to the GPIO pins in the bus.
    /// The values are written as a byte, LSb first.
    pub fn write_byte(&self, value: u8) -> GpioResult<()> {
        let mut values = [false; 8];
        for i in 0..8 {
            values[i] = (value & (1 << i)) != 0;
        }
        self.write(&values)
    }
}

impl dyn GpioBusOutput<4> + '_ {
    /// Writes the values to the GPIO pins in the bus.
    /// The values are written as a nibble, LSb first.
    pub fn write_nibble(&self, value: u8) -> GpioResult<()> {
        if value > 0b1111 {
            return Err(GpioError::InvalidArgument);
        }

        let mut values = [false; 4];
        for i in 0..4 {
            values[i] = (value & (1 << i)) != 0;
        }
        self.write(&values)
    }
}
