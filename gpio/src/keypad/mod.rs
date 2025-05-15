mod gpio;

use std::fmt::Debug;
use crate::GpioResult;
pub use gpio::*;

pub trait Keypad: Debug {
    type Key;

    fn read(&self) -> GpioResult<Vec<Self::Key>>;
}
