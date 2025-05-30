pub mod gpiod;
pub mod lcd;
pub mod debounce;
pub mod pwm;
pub mod raw;
pub mod clock;
pub mod keypad;
pub mod soft;
pub mod rotenc;

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

/// Specifies the active level of the GPIO pin.
///
/// By default, the active level is high.
///
/// Might be software-implemented.
#[derive(Copy, Clone, Debug, Default)]
pub enum GpioActiveLevel {
    #[default] High,
    Low,
}

impl GpioActiveLevel {
    /// Gets the real state that will be outputted on the GPIO pin based on the active level and the value.
    pub fn get_state(&self, value: bool) -> bool {
        match self {
            GpioActiveLevel::High => value,
            GpioActiveLevel::Low => !value,
        }
    }
}


/// Specifies the bias of the GPIO pin.
///
/// You can use this to enable pull-up or pull-down resistors.
/// These should work in both input and output modes.
#[derive(Copy, Clone, Debug, Default)]
pub enum GpioBias {
    #[default] None,
    PullUp,
    PullDown,
}

/// Specifies the drive mode of the GPIO pin.
///
/// Works only in output mode.
///
/// By default, the drive mode is push-pull, which drives the pin high or low with low impedance.
/// There's also open-drain and open-source modes, that leave the pin floating when the output is high or low, respectively.
///
/// Leaving the pin floating might be implemented by setting the pin to input mode.
#[derive(Copy, Clone, Debug, Default)]
pub enum GpioDriveMode {
    /// GPIO pin is driven high or low with low impedance.
    #[default] PushPull,
    /// GPIO pin is driven low or left floating when high.
    OpenDrain,
    /// GPIO pin is driven high or left floating when low.
    OpenSource,
}

impl GpioDriveMode {
    /// Gets the real state that will be outputted on the GPIO pin based on the drive mode and the value.
    ///
    /// # Returns
    /// - `Some(true)` if the pin will be driven high.
    /// - `Some(false)` if the pin will be driven low.
    /// - `None` if the pin will be left floating.
    pub fn get_state(&self, value: bool) -> Option<bool> {
        match self {
            GpioDriveMode::PushPull => Some(value),
            GpioDriveMode::OpenDrain => if value { None } else { Some(false) },
            GpioDriveMode::OpenSource => if value { Some(true) } else { None },
        }
    }
}

pub trait GpioPin: Debug {
    /// Sets the GPIO pin function to input, allowing reading its state.
    fn as_input(&mut self) -> GpioResult<Box<dyn GpioInput + '_>>;
    /// Sets the GPIO pin function to output, allowing writing its state.
    fn as_output(&mut self) -> GpioResult<Box<dyn GpioOutput + '_>>;

    /// Gets whether the GPIO pin supports active level.
    fn supports_active_level(&self) -> bool {
        false
    }
    /// Gets the active level of the GPIO pin.
    fn active_level(&self) -> GpioActiveLevel {
        GpioActiveLevel::High
    }
    /// Sets the active level of the GPIO pin.
    ///
    /// # Errors
    /// - `GpioError::NotSupported` if the pin does not support active level.
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

    /// Gets whether the GPIO pin supports bias (pull-up/pull-down resistors).
    fn supports_bias(&self) -> bool {
        false
    }
    /// Gets the bias of the GPIO pin.
    fn bias(&self) -> GpioBias {
        GpioBias::None
    }
    /// Sets the bias of the GPIO pin.
    ///
    /// # Errors
    /// - `GpioError::NotSupported` if the pin does not support bias.
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

    /// Gets whether the GPIO pin supports drive mode (push-pull, open-drain, open-source).
    fn supports_drive_mode(&self) -> bool {
        false
    }
    /// Gets the drive mode of the GPIO pin.
    fn drive_mode(&self) -> GpioDriveMode {
        GpioDriveMode::PushPull
    }
    /// Sets the drive mode of the GPIO pin.
    ///
    /// # Errors
    /// - `GpioError::NotSupported` if the pin does not support drive mode.
    fn set_drive_mode(&mut self, _mode: GpioDriveMode) -> GpioResult<()> {
        Err(GpioError::NotSupported)
    }

    fn with_drive_mode(mut self, mode: GpioDriveMode) -> GpioResult<Self>
    where
        Self: Sized,
    {
        self.set_drive_mode(mode)?;
        Ok(self)
    }
}

pub trait GpioInput: Debug {
    /// Reads the state of the GPIO pin.
    fn read(&self) -> GpioResult<bool>;
}

pub trait GpioOutput: Debug {
    /// Writes the state of the GPIO pin.
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

    fn supports_drive_mode(&self) -> bool {
        false
    }
    fn drive_mode(&self) -> GpioDriveMode {
        GpioDriveMode::PushPull
    }

    fn set_drive_mode(&mut self, _mode: GpioDriveMode) -> GpioResult<()> {
        Err(GpioError::NotSupported)
    }

    fn with_drive_mode(mut self, mode: GpioDriveMode) -> GpioResult<Self>
    where
        Self: Sized,
    {
        self.set_drive_mode(mode)?;
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
