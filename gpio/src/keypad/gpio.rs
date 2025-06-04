use std::fmt::{Debug, Formatter};
use crate::{GpioBusInput, GpioBusOutput, GpioResult};
use crate::keypad::Keypad;

/// Represents the keys on a 4x4 keypad.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeypadKey {
    /// The `1` key.
    Key1,
    /// The `2` key.
    Key2,
    /// The `3` key.
    Key3,
    /// The `4` key.
    Key4,
    /// The `5` key.
    Key5,
    /// The `6` key.
    Key6,
    /// The `7` key.
    Key7,
    /// The `8` key.
    Key8,
    /// The `9` key.
    Key9,
    /// The `0` key.
    Key0,
    /// The `*` key.
    KeyAsterisk,
    /// The `#` key.
    KeyHash,
    /// The `A` key.
    KeyA,
    /// The `B` key.
    KeyB,
    /// The `C` key.
    KeyC,
    /// The `D` key.
    KeyD,
}

impl KeypadKey {
    /// Converts a position tuple (row, column) to a [KeypadKey].
    pub fn from_position(pos: (u8, u8)) -> Option<KeypadKey> {
        use KeypadKey::*;

        const KEYS: [[KeypadKey; 4]; 4] = [
            [ Key1, Key2, Key3, KeyA, ],
            [ Key4, Key5, Key6, KeyB, ],
            [ Key7, Key8, Key9, KeyC, ],
            [ KeyAsterisk, Key0, KeyHash, KeyD, ],
        ];

        if pos.0 < 4 && pos.1 < 4 {
            Some(KEYS[pos.0 as usize][pos.1 as usize])
        } else {
            None
        }
    }

    /// Converts the [KeypadKey] to its corresponding character.
    pub fn to_char(self) -> char {
        use KeypadKey::*;

        match self {
            Key1 => '1',
            Key2 => '2',
            Key3 => '3',
            Key4 => '4',
            Key5 => '5',
            Key6 => '6',
            Key7 => '7',
            Key8 => '8',
            Key9 => '9',
            Key0 => '0',
            KeyAsterisk => '*',
            KeyHash => '#',
            KeyA => 'A',
            KeyB => 'B',
            KeyC => 'C',
            KeyD => 'D',
        }
    }
}

/// The `GpioKeypad` struct represents a GPIO-based keypad with 4 columns and 4 rows.
pub struct GpioKeypad<'a> {
    cols: &'a dyn GpioBusOutput<4>,
    rows: &'a dyn GpioBusInput<4>,
}

impl Debug for GpioKeypad<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GpioKeypad({:?}, {:?})", self.cols, self.rows)
    }
}

impl <'a> GpioKeypad<'a> {
    /// Creates a new `GpioKeypad` instance with the specified GPIO bus outputs for columns and inputs for rows.
    /// 
    /// The columns are used to write the active state, and the rows are used to read the pressed keys
    /// in the active column.
    pub fn new(cols: &'a dyn GpioBusOutput<4>, rows: &'a dyn GpioBusInput<4>) -> Self {
        GpioKeypad { cols, rows }
    }
}

impl Keypad for GpioKeypad<'_> {
    type Key = KeypadKey;

    fn read(&self) -> GpioResult<Vec<Self::Key>> {
        let mut pressed = Vec::new();

        for col in 0..4 {
            let nibble = 1 << (3 - col);
            self.cols.write_nibble(nibble)?;
            let value = self.rows.read_nibble()?;
            for row in 0..4 {
                let value = value >> (3 - row) & 1;
                if value == 1 {
                    if let Some(key) = KeypadKey::from_position((row, col)) {
                        pressed.push(key);
                    }
                }
            }
        }

        Ok(pressed)
    }
}
