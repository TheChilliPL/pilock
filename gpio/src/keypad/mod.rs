mod gpio;

use std::fmt::Debug;
use crate::GpioResult;
pub use gpio::*;

/// The `Keypad` trait defines the interface for keypad input devices.
pub trait Keypad: Debug {
    type Key;

    fn read(&self) -> GpioResult<Vec<Self::Key>>;
}
