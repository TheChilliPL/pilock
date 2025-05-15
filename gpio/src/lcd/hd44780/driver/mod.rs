mod gpio;

use crate::{GpioError, GpioResult};
pub use gpio::*;
use std::fmt::Debug;

pub trait HD44780Driver: Debug {
    /// Initializes the HD44780 controller with the default settings.
    fn init(&mut self, multiline: bool, alt_font: bool) -> GpioResult<()>;

    /// Clears the display and sets the cursor to the home position.
    fn clear_display(&mut self) -> GpioResult<()> {
        self.send_command(0b00000001)
    }

    /// Sets the cursor to the home position.
    fn return_home(&mut self) -> GpioResult<()> {
        self.send_command(0b00000010)
    }

    /// Sets the display to the specified entry mode.
    fn set_entry_mode(&mut self, cursor_direction: CursorDirection, shift: bool) -> GpioResult<()> {
        let mut command = 0b00000100;
        if cursor_direction == CursorDirection::Right {
            command |= 0b00000010;
        }
        if shift {
            command |= 0b00000001;
        }
        self.send_command(command)
    }

    /// Sets the display on/off, cursor on/off, and blinking on/off.
    fn set_display_control(
        &mut self,
        display_on: bool,
        cursor_on: bool,
        blink_on: bool,
    ) -> GpioResult<()> {
        let mut command = 0b00001000;
        if display_on {
            command |= 0b00000100;
        }
        if cursor_on {
            command |= 0b00000010;
        }
        if blink_on {
            command |= 0b00000001;
        }
        self.send_command(command)
    }

    /// Moves the cursor or shifts the display.
    fn cursor_shift(&mut self, display_shift: bool, direction: CursorDirection) -> GpioResult<()> {
        let mut command = 0b00010000;
        if display_shift {
            command |= 0b00001000;
        }
        if direction == CursorDirection::Right {
            command |= 0b00000100;
        }
        self.send_command(command)
    }

    /// Sets the function set.
    fn function_set(&mut self, data_length: bool, two_lines: bool, font: bool) -> GpioResult<()> {
        let mut command = 0b00100000;
        if data_length {
            command |= 0b00010000;
        }
        if two_lines {
            command |= 0b00001000;
        }
        if font {
            command |= 0b00000100;
        }
        self.send_command(command)
    }

    /// Sets the CGRAM address.
    fn set_cgram_address(&mut self, address: u8) -> GpioResult<()> {
        if address > 0b00111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b01000000 | address;
        self.send_command(command)
    }

    /// Sets the DDRAM address.
    fn set_ddram_address(&mut self, address: u8) -> GpioResult<()> {
        if address > 0b01111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b10000000 | address;
        self.send_command(command)
    }

    /// Reads the busy flag and address counter.
    fn get_busy_flag_and_address(&mut self) -> GpioResult<(bool, u8)> {
        let command = self.read_command()?;
        let busy_flag = command & 0b10000000 != 0;
        let address = command & 0b01111111;
        Ok((busy_flag, address))
    }

    // Low-level commands
    // These raw commands are used by the high-level functions above.
    // They are not meant to be used directly, but implemented by the driver implementation.

    /// Sends a command to the HD44780 controller.
    /// Sets the RS pin to 0 (command).
    fn send_command(&mut self, command: u8) -> GpioResult<()>;

    /// Sends data to the HD44780 controller.
    /// Sets the RS pin to 1 (data).
    fn send_data(&mut self, data: u8) -> GpioResult<()>;

    /// Reads the busy flag and address counter.
    /// Sets the RS pin to 0 (command).
    ///
    /// Returns both in a single u8, for easier usage use [Self::get_busy_flag_and_address], which
    /// uses this function internally.
    fn read_command(&mut self) -> GpioResult<u8>;

    /// Reads data from the HD44780 controller.
    /// Sets the RS pin to 1 (data).
    fn read_data(&mut self) -> GpioResult<u8>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CursorDirection {
    /// Moves the cursor to the left after writing/reading data.
    Left,
    /// Moves the cursor to the right after writing/reading data.
    Right,
}
