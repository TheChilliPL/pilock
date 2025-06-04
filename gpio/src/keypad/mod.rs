//! Keypad module.
//! 
//! Not used finally due to the physical keypad being broken. Replaced by the [rotary encoder](crate::rotenc).
//! The keypad module itself works correctly, but the physical keypad was not reliable.

mod gpio;

use std::fmt::Debug;
use crate::GpioResult;
pub use gpio::*;

/// The `Keypad` trait defines the interface for keypad input devices.
/// 
/// For the specific implementation, see [GpioKeypad].
pub trait Keypad: Debug {
    /// The type of key that the keypad can read.
    type Key;

    /// Reads the current state of the keypad and returns a vector of keys that are currently pressed.
    fn read(&self) -> GpioResult<Vec<Self::Key>>;
}
