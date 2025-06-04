//! SSD1803A LCD driver module.
//!
//! See [SSD1803ADriver] trait for detailed documentation of the driver interface, and [GpioSSD1803ADriver]
//! for the implementation of the driver using GPIO pins. This trait is designed to be compatible with
//! other protocols like I2C or SPI in the future.

mod gpio;

use std::fmt::Debug;
use crate::{GpioError, GpioResult};
use crate::lcd::hd44780::driver::{CursorDirection, HD44780Driver};
pub use gpio::*;

/// The `SSD1803ADriver` trait defines a low-level interface for SSD1803A LCD controller drivers, like the
/// DOGM204-A used in this project.
///
/// # Extended registers and HD44780 compatibility
///
/// This LCD controller is mostly compatible with the HD44780 controller, but has some extra features.
/// The additional features use the reserved bits in the HD44780 command set. For compatibility with
/// HD44780 commands, these bits **have to** be set to `0` when sending commands that are not
/// SSD1803A-specific.
///
/// Additionally, the driver has two “hidden” bits — `RE` and `IS`, called “Extended function registers bit”
/// and “Special registers enable bit” respectively by the SSD1803A documentation. These bits are
/// set using the [SSD1803ADriver::function_set_0] and [SSD1803ADriver::function_set_1] methods.
/// See these for more details.
///
/// Some HD44780 commands might require these bits to be set to `0` to work correctly, while some
/// SSD1803A-specific commands require them to be set to `1`. The driver will ensure that these bits
/// are set correctly before sending the actual command by using [SSD1803ADriver::send_command],
/// and otherwise call the aforementioned function set methods itself.
///
/// # Memory
///
/// The SSD1803A controller has several built-in memory areas:
/// - **CGROM** (Character Generator ROM) — there are 3 ROMs available, each with a different set of characters.
///   In contrast to the HD44780, all the ROMs are available in each display. They can be switched
///   using the [SSD1803ADriver::rom_selection] command. See chapter 15 of the SSD1803A documentation
///   (pages 66–68) for the full tables of characters.
/// - **CGRAM** (Character Generator RAM) — this is a user-defined character generator memory, which can
///   store up to 8 user-defined characters. Each character is 5x8 pixels, and can be defined
///   by writing data to the CGRAM.
/// - **DDRAM** (Display Data RAM) — this is the main display memory, which stores the characters to be displayed.
/// - **SEGRAM** (Segment RAM) — this is a special memory area used for the icon display. Sadly, icon
///   display is barely mentioned in the documentation, and I couldn't get it to work nor find any
///   specific information about it.
///
/// # Commands
///
/// Each command is a byte. In the documentation, the bits that can be set to anything are marked as
/// `?`. In case of the data bits, they're set to `0` by default, and in case of the IS and RE bits,
/// they're only set to `0` or `1` using the function set commands when needed.
/// See all the various methods in this trait for the commands that can be sent to the SSD1803A controller.
///
/// # Sources
///
/// - Display Visions Industrial Solutions, [“DOGM204-A 4x20 INCL. CONTROLLER SSD1803A for 4-/8 Bit,
///   SPI, I²C,”](https://www.lcd-module.com/fileadmin/eng/pdf/doma/dogm204e.pdf) Mar. 2022,
/// - Solomon Systech Limited,
///   [“SSD1803A LCD Segment / Common Mono Driver with Controller,”](https://www.lcd-module.de/fileadmin/eng/pdf/zubehoer/ssd1803a_2_0.pdf)
///   May 2011.
pub trait SSD1803ADriver: Debug {
    /// Initializes the SSD1803A controller with the default settings, using the sequence mentioned
    /// in the documentation of the controller. This is to be implemented by the specific driver
    /// implementation, so see [GpioSSD1803ADriver::init] for more information.
    ///
    /// The `lines` parameter specifies the number of lines on the display. SSD1803A supports up to 4 lines.
    fn init(&mut self, lines: u8) -> GpioResult<()>;

    /// Clears the display and sets the cursor to the home position.
    ///
    /// Command: `00000001`, IS: `?`, RE: `?`.
    /// HD44780-compatible.
    fn clear_display(&mut self) -> GpioResult<()> {
        self.send_command(0b00000001, None, None)
    }

    /// Sets the cursor to the home position (0, 0).
    ///
    /// Command: `0000001?`, IS: `?`, RE: `0`.
    /// HD44780-compatible.
    fn return_home(&mut self) -> GpioResult<()> {
        self.send_command(0b00000010, None, Some(false))
    }

    /// Sets the display to power-down mode. Can be used to save power when the display is not in use.
    /// According to the documentation, it disables the built-in voltage converter.
    ///
    /// Command: `0000001P`, IS: `?`, RE: `1`.
    /// `P` is `1` for power-down mode, `0` for normal operation.
    fn power_down_mode(&mut self, power_down: bool) -> GpioResult<()> {
        let mut command = 0b00000010;
        if power_down {
            command |= 0b00000001;
        }
        self.send_command(command, None, Some(true))
    }

    /// Sets the entry mode for the display, which controls how the cursor moves and whether the display is shifted.
    ///
    /// Command: `000001IS`, IS: `?`, RE: `0`.
    /// `I` is `1` for right cursor direction, `0` for left cursor direction.
    /// `S` is `1` for display shift, `0` for no display shift.
    /// HD44780-compatible.
    fn set_entry_mode(&mut self, cursor_direction: CursorDirection, shift: bool) -> GpioResult<()> {
        let mut command = 0b00000100;
        if cursor_direction == CursorDirection::Right {
            command |= 0b00000010;
        }
        if shift {
            command |= 0b00000001;
        }
        self.send_command(command, None, Some(false))
    }

    /// Sets data shift direction of display segments. This can flip the whole display horizontally or vertically,
    /// allowing it to be used upside down.
    ///
    /// Command: `000001CS`, IS: `?`, RE: `1`.
    /// `C` is `1` for reverse COM direction (vertical flip), `0` for normal COM direction.
    /// `S` is `1` for reverse SEG direction (horizontal flip), `0` for normal SEG direction.
    fn set_entry_mode_ex(&mut self, reverse_bds: bool, reverse_bdc: bool) -> GpioResult<()> {
        let mut command = 0b00000100;
        if reverse_bdc {
            command |= 0b00000010;
        }
        if reverse_bds {
            command |= 0b00000001;
        }
        self.send_command(command, None, Some(true))
    }

    /// Turns the display on or off, and controls the cursor and its blinking.
    /// This does not shut down the ram or the voltage converter, so the display will still consume power.
    ///
    /// Command: `00001DCB`, IS: `?`, RE: `0`.
    /// `D` is `1` for display on, `0` for display off.
    /// `C` is `1` for cursor on, `0` for cursor off.
    /// `B` is `1` for cursor blinking, `0` for cursor not blinking.
    fn set_display_control(&mut self, display_on: bool, cursor_on: bool, blink_on: bool) -> GpioResult<()> {
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
        self.send_command(command, None, Some(false))
    }

    /// Extended function set.
    /// Controls font width (6-bit or 5-bit (default)), cursor inversion, and number of lines.
    ///
    /// Command: `00001FBL`, IS: `?`, RE: `1`.
    /// `F` is `1` for wide font (6-bit), `0` for narrow font (5-bit).
    /// `B` is `1` for inverted cursor, `0` for normal cursor.
    /// `L` is `1` for 3 or 4 lines, `0` for 1 or 2 lines. This has to be coordinated with the lines number in [SSD1803ADriver::function_set_0].
    ///
    fn ext_function_set(
        &mut self,
        wide_font: bool,
        invert_cursor: bool,
        lines_3_or_4: bool,
    ) -> GpioResult<()> {
        let mut command = 0b00001000;
        if wide_font {
            command |= 0b00000100;
        }
        if invert_cursor {
            command |= 0b00000010;
        }
        if lines_3_or_4 {
            command |= 0b00000001;
        }
        self.send_command(command, None, Some(true))
    }

    /// Shifts the cursor or the display by one in the specified direction.
    ///
    /// Command: `0001DR??`, IS: `0`, RE: `0`.
    /// `D` is `1` for display shift, `0` for cursor shift.
    /// `R` is `1` for right shift, `0` for left shift.
    /// HD44780-compatible.
    fn cursor_shift(&mut self, display_shift: bool, direction: CursorDirection) -> GpioResult<()> {
        let mut command = 0b00010000;
        if display_shift {
            command |= 0b00001000;
        }
        if direction == CursorDirection::Right {
            command |= 0b00000100;
        }
        self.send_command(command, Some(false), Some(false))
    }

    /// Sets the double height mode, bias divider, and dot shift.
    ///
    /// Command: `0001UUBD`, IS: `0`, RE: `1`.
    /// `UU` is the double height mode, where:
    /// - `00` is single-single-double,
    /// - `01` is single-double-single,
    /// - `10` is double-double,
    /// - `11` is double-single-single.
    ///
    /// It makes the lines marked as `double` twice as high as the others, by vertically stretching the characters.
    /// It has to be turned on by [SSD1803ADriver::function_set_0].
    /// `B` sets the `BS1` bit of the bias divider, which is used along with [SSD1803ADriver::internal_osc_frequency]
    /// to set the bias divider:
    ///
    /// | BS1 | BS0 | Bias          |
    /// |-----|-----|---------------|
    /// | 0   | 0   | 1/5 (default) |
    /// | 0   | 1   | 1/4           |
    /// | 1   | 0   | 1/7           |
    /// | 1   | 1   | 1/6           |
    ///
    /// `D` is `1` for smooth dot shift, `0` for normal display shift per line.
    fn double_height_bias_dot_shift(
        &mut self,
        double_height: DoubleHeightMode,
        bias_divider: BiasDivider,
        dot_shift: bool,
    ) -> GpioResult<()> {
        let mut command = 0b00010000;
        command |= double_height.to_mask();
        if bias_divider.bs1() {
            command |= 0b00000010;
        }
        if !dot_shift { // This is inverted!!
            command |= 0b00000001;
        }
        self.send_command(command, Some(false), Some(true))
    }

    /// Sets the internal oscillator frequency and bias divider.
    ///
    /// Command: `0001BFFF`, IS: `1`, RE: `0`.
    /// `B` sets the `BS0` bit of the bias divider, which is used along with [SSD1803ADriver::double_height_bias_dot_shift]
    /// to set the bias divider:
    ///
    /// | BS1 | BS0 | Bias          |
    /// |-----|-----|---------------|
    /// | 0   | 0   | 1/5 (default) |
    /// | 0   | 1   | 1/4           |
    /// | 1   | 0   | 1/7           |
    /// | 1   | 1   | 1/6           |
    ///
    /// `FFF` is the internal oscillator frequency, which can be one of the following:
    ///
    /// | FFF | Frequency |
    /// |-----|-----------|
    /// | 111 | 680 kHz   |
    /// | 110 | 640 kHz   |
    /// | 101 | 620 kHz   |
    /// | 100 | 580 kHz   |
    /// | 011 | 540 kHz (default) |
    /// | 010 | 500 kHz   |
    /// | 001 | 460 kHz   |
    /// | 000 | 420 kHz   |
    fn internal_osc_frequency(
        &mut self,
        bias_divider: BiasDivider,
        frequency: InternalOscFrequency,
    ) -> GpioResult<()> {
        let mut command = 0b00010000;
        if bias_divider.bs0() {
            command |= 0b00001000;
        }
        command |= frequency.to_mask();
        self.send_command(command, Some(true), Some(false))
    }

    /// Enables or disables shifting specific lines of the display.
    ///
    /// Command: `0001SSSS`, IS: `1`, RE: `1`.
    /// `SSSS` is a 4-bit value where each bit corresponds to one line of the display, in reverse order.
    /// Value of `1` means the line is shifted, `0` means it is not.
    fn shift_enable(
        &mut self,
        shift_1: bool,
        shift_2: bool,
        shift_3: bool,
        shift_4: bool,
    ) -> GpioResult<()> {
        let mut command = 0b00010000;
        if shift_4 {
            command |= 0b00001000;
        }
        if shift_3 {
            command |= 0b00000100;
        }
        if shift_2 {
            command |= 0b00000010;
        }
        if shift_1 {
            command |= 0b00000001;
        }
        self.send_command(command, Some(true), Some(true))
    }

    /// Sets the function set for the SSD1803A controller.
    ///
    /// Command: `001BLD0I`, IS: `?`, RE: `?`.
    /// `B` is `1` for 8-bit data length, `0` for 4-bit data length.
    /// `L` is `1` for 2 or 4 lines, `0` for 1 or 3 lines.
    /// `D` is `1` for double height enabled, according to the set double height mode.
    /// **`RE` bit register will be set to `0`**.
    /// `I` is the value that the **`IS` bit register will be set to**.
    /// HD44780-compatible.
    fn function_set_0(
        &mut self,
        data_length: bool,
        lines_2_or_4: bool,
        double_height_enabled: bool,
        set_is: bool,
    ) -> GpioResult<()> {
        let mut command = 0b00100000;
        if data_length {
            command |= 0b00010000;
        }
        if lines_2_or_4 {
            command |= 0b00001000;
        }
        if double_height_enabled {
            command |= 0b00000100;
        }
        if set_is {
            command |= 0b00000001;
        }
        self.send_command(command, None, None)
    }

    /// Sets the function set for the SSD1803A controller, with additional options for character blinking and inversion.
    ///
    /// Command: `001BLb1r`, IS: `?`, RE: `?`.
    /// `B` is `1` for 8-bit data length, `0` for 4-bit data length.
    /// `L` is `1` for 2 or 4 lines, `0` for 1 or 3 lines.
    /// `b` is `1` for user character blinking enabled, `0` for disabled.
    /// **`RE` bit register will be set to `1`**.
    /// `r` inverts the display if set to `1`.
    fn function_set_1(
        &mut self,
        data_length: bool,
        lines_2_or_4: bool,
        char_blink: bool,
        invert: bool,
    ) -> GpioResult<()> {
        let mut command = 0b00100010;
        if data_length {
            command |= 0b00010000;
        }
        if lines_2_or_4 {
            command |= 0b00001000;
        }
        if char_blink {
            command |= 0b00000100;
        }
        if invert {
            command |= 0b00000001;
        }
        self.send_command(command, None, None)
    }

    /// Sets the cursor to the specified CGRAM address (custom character memory).
    ///
    /// The address is a 6-bit value. If out of bounds, it will return [GpioError::InvalidArgument].
    ///
    /// Command: `01AAAAAA`, IS: `0`, RE: `0`.
    /// `AAAAAA` is the address.
    /// HD44780-compatible.
    fn set_cgram_address(&mut self, address: u8) -> GpioResult<()> {
        if address > 0b00111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b01000000 | address;
        self.send_command(command, Some(false), Some(false))
    }

    /// Sets the address of the SEGRAM (Segment RAM) for the icon display. As mentioned in the trait
    /// documentation, I couldn't find any specific information about the icon display, nor get it
    /// to work.
    ///
    /// The address is a 4-bit value, and if out of bounds, it will return [GpioError::InvalidArgument].
    ///
    /// Command: `0100AAAA`, IS: `1`, RE: `0`.
    /// `AAAA` is the address.
    fn set_segram_address(&mut self, address: u8) -> GpioResult<()> {
        if address > 0b00001111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b01000000 | address;
        self.send_command(command, Some(true), Some(false))
    }

    /// Sets the icon display mode, switches voltage converter and regulator circuit, and sets the two
    /// highest bytes of the contrast.
    ///
    /// Command: `0101IBCC`, IS: `1`, RE: `0`.
    /// `I` is `1` for icon display enabled, `0` for disabled.
    /// `B` is `1` for booster regulator circuit enabled, `0` for disabled.
    /// `CC` is the two highest bits of the contrast value, which is a 6-bit value.
    fn icon_booster_contrast(
        &mut self,
        icon_display: bool,
        booster_regulator_on: bool,
        contrast: u8,
    ) -> GpioResult<()> {
        if contrast > 0b00111111 {
            return Err(GpioError::InvalidArgument);
        }
        let mut command = 0b01010000;
        if icon_display {
            command |= 0b00001000;
        }
        if booster_regulator_on {
            command |= 0b00000100;
        }
        command |= contrast >> 4;
        self.send_command(command, Some(true), Some(false))
    }

    /// Controls the follower circuit, which is used for the voltage divider circuit and internal resistor ratio.
    ///
    /// Command: `0110DRRR`, IS: `1`, RE: `0`.
    /// `D` is `1` for divider circuit enabled, `0` for disabled.
    /// `RRR` is the internal resistor ratio. For more information, look at page 48 of the
    /// [SSD1803A documentation](https://www.lcd-module.de/fileadmin/eng/pdf/zubehoer/ssd1803a_2_0.pdf).
    fn follower_control(
        &mut self,
        divider_circuit: bool,
        internal_resistor_ratio: InternalResistorRatio,
    ) -> GpioResult<()> {
        let mut command = 0b01100000;
        if divider_circuit {
            command |= 0b00001000;
        }
        command |= internal_resistor_ratio.to_mask();
        self.send_command(command, Some(true), Some(false))
    }

    /// Sets the 4 lowest bits of the contrast value, which is a 6-bit value.
    ///
    /// Command: `0111CCCC`, IS: `1`, RE: `0`.
    /// `CCCC` is the 4 lowest bits of the contrast value.
    fn contrast_set(&mut self, contrast: u8) -> GpioResult<()> {
        if contrast > 0b00111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b01110000 | (contrast & 0b1111);
        self.send_command(command, Some(true), Some(false))
    }

    /// Sets the DDRAM (Display Data RAM) address, which is used to set the cursor position.
    ///
    /// With the 4-lines display, the lines start at `0x00`, `0x20`, `0x40`, and `0x60` respectively.
    /// The address is a 7-bit value, and if out of bounds, it will return [GpioError::InvalidArgument].
    ///
    /// Command: `1AAAAAAA`, IS: `?`, RE: `0`.
    /// `AAAAAAA` is the address.
    fn set_ddram_address(&mut self, address: u8) -> GpioResult<()> {
        if address > 0b01111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b10000000 | address;
        self.send_command(command, None, Some(false))
    }

    /// Sets the scroll quantity for the dot shift, up to 48 dots.
    ///
    /// Command: `1?SSSSSS`, IS: `?`, RE: `1`.
    /// `SSSSSS` is the scroll quantity.
    fn set_scroll_quantity(&mut self, quantity: u8) -> GpioResult<()> {
        if quantity > 0b00111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b10000000 | quantity;
        self.send_command(command, None, Some(true))
    }

    /// Reads the busy flag and the current address from the controller.
    ///
    /// This is done by writing data with RS set to `0` and reading the data pins.
    ///
    /// **⚠️ This is *mostly* compatible with the HD44780, but if called twice, it will return
    /// the part ID instead of the address!**
    fn get_busy_flag_and_address(&mut self) -> GpioResult<(bool, u8)> {
        let command = self.read_command()?;
        let busy_flag = command & 0b10000000 != 0;
        let address = command & 0b01111111;
        Ok((busy_flag, address))
    }

    /// Sets the temperature coefficient for the display.
    ///
    /// This is a special command as it requires sending **two** bytes, with `RE` set to 1:
    /// `01110110` `00000TTT`, where `TTT` is the temperature coefficient:
    /// 
    /// - `010` for -0.05%/°C,
    /// - `100` for -0.10%/°C,
    /// - `110` for -0.15%/°C,
    /// - `111` for -0.20%/°C,
    /// 
    /// and all the other values reserved.
    /// RS must be set to `0` for the first byte, and `1` for the second byte.
    fn temp_coefficient_control(
        &mut self,
        temp_coefficient: TempCoefficient,
    ) -> GpioResult<()> {
        self.send_command(0b01110110, None, Some(true))?;
        self.send_data(temp_coefficient.to_mask())
    }

    /// Sets the ROM selection for the SSD1803A controller.
    ///
    /// This is a special command as it requires sending **two** bytes, with `RE` set to 1:
    /// `01110010` `0000RR00`, where `RR` is the ROM selection:
    /// 
    /// - `00` for ROM A,
    /// - `01` for ROM B,
    /// - `10` for ROM C,
    /// 
    /// and `11` is reserved. See the trait documentation for more information.
    /// RS must be set to `0` for the first byte, and `1` for the second byte.
    fn rom_selection(
        &mut self,
        rom: Rom,
    ) -> GpioResult<()> {
        self.send_command(0b01110010, None, Some(true))?;
        self.send_data(rom.to_mask())
    }

    /// Sends a command to the SSD1803A controller. RS pin is set to `0` for command mode.
    ///
    /// Before sending the raw command, ensures IS and RE bits are set correctly,
    /// and sets them otherwise by calling the appropriate function set methods.
    fn send_command(&mut self, data: u8, is: Option<bool>, re: Option<bool>) -> GpioResult<()>;

    /// Sends data to the SSD1803A controller. RS pin is set to `1` for data mode.
    fn send_data(&mut self, data: u8) -> GpioResult<()>;

    /// Reads a command from the SSD1803A controller. RS pin is set to `0` for command mode.
    fn read_command(&mut self) -> GpioResult<u8>;

    /// Reads data from the SSD1803A controller. RS pin is set to `1` for data mode.
    fn read_data(&mut self) -> GpioResult<u8>;
}

/// Compatibility layer allowing the use of SSD1803A driver as an HD44780 driver.
impl HD44780Driver for dyn SSD1803ADriver {
    /// Initializes the HD44780 controller with the default settings.
    ///
    /// ⚠️ Alternative font is not supported by the SSD1803A controller.
    fn init(&mut self, multiline: bool, _alt_font: bool) -> GpioResult<()> {
        self.init(if multiline { 2 } else { 1 })
    }

    fn send_command(&mut self, command: u8) -> GpioResult<()> {
        self.send_command(command, Some(false), Some(false))
    }

    fn send_data(&mut self, data: u8) -> GpioResult<()> {
        self.send_data(data)
    }

    fn read_command(&mut self) -> GpioResult<u8> {
        self.read_command()
    }

    fn read_data(&mut self) -> GpioResult<u8> {
        self.read_data()
    }
}

/// The double height mode for the SSD1803A controller.
#[derive(Clone, Copy, Debug, Default)]
pub enum DoubleHeightMode {
    /// Single-Single-Double
    DoubleBottom,
    /// Single-Double-Single
    DoubleMiddle,
    /// Double-Double
    DoubleBoth,
    #[default]
    /// Double-Single-Single
    DoubleTop,
}

impl DoubleHeightMode {
    /// Converts the double height mode to a mask byte for the command.
    pub fn to_mask(&self) -> u8 {
        match self {
            DoubleHeightMode::DoubleBottom => 0b00000000,
            DoubleHeightMode::DoubleMiddle => 0b00000100,
            DoubleHeightMode::DoubleBoth =>   0b00001000,
            DoubleHeightMode::DoubleTop =>    0b00001100,
        }
    }
}

/// The bias divider for the SSD1803A controller.
#[derive(Clone, Copy, Debug, Default)]
pub enum BiasDivider {
    /// 1/5 (default for SSD1803A controller)
    Bias1_5,
    /// 1/4
    Bias1_4,
    /// 1/7
    Bias1_7,
    #[default]
    /// 1/6 (default for DOGM204-A display)
    Bias1_6,
}

impl BiasDivider {
    /// Gets the BS1 bit of the bias divider.
    pub fn bs1(&self) -> bool {
        match self {
            BiasDivider::Bias1_5 => false,
            BiasDivider::Bias1_4 => false,
            BiasDivider::Bias1_7 => true,
            BiasDivider::Bias1_6 => true,
        }
    }

    /// Gets the BS0 bit of the bias divider.
    pub fn bs0(&self) -> bool {
        match self {
            BiasDivider::Bias1_5 => false,
            BiasDivider::Bias1_4 => true,
            BiasDivider::Bias1_7 => false,
            BiasDivider::Bias1_6 => true,
        }
    }
}

/// The internal oscillator frequency of the SSD1803A controller.
#[derive(Clone, Copy, Debug, Default)]
pub enum InternalOscFrequency {
    /// 420 kHz
    Freq420kHz,
    /// 460 kHz
    Freq460kHz,
    /// 500 kHz
    Freq500kHz,
    /// 540 kHz (default)
    #[default] Freq540kHz,
    /// 580 kHz
    Freq580kHz,
    /// 620 kHz
    Freq620kHz,
    /// 640 kHz
    Freq640kHz,
    /// 680 kHz
    Freq680kHz,
}

impl InternalOscFrequency {
    /// Converts the internal oscillator frequency to a mask byte for the command.
    pub fn to_mask(&self) -> u8 {
        match self {
            InternalOscFrequency::Freq420kHz => 0b00000000,
            InternalOscFrequency::Freq460kHz => 0b00000001,
            InternalOscFrequency::Freq500kHz => 0b00000010,
            InternalOscFrequency::Freq540kHz => 0b00000011,
            InternalOscFrequency::Freq580kHz => 0b00000100,
            InternalOscFrequency::Freq620kHz => 0b00000101,
            InternalOscFrequency::Freq640kHz => 0b00000110,
            InternalOscFrequency::Freq680kHz => 0b00000111,
        }
    }
}

/// The internal resistor ratio for the SSD1803A controller.
#[derive(Clone, Copy, Debug, Default)]
pub enum InternalResistorRatio {
    /// 1 + Rb / Ra = 1.9
    IR0,
    /// 1 + Rb / Ra = 2.2
    IR1,
    /// 1 + Rb / Ra = 2.6
    #[default] IR2,
    /// 1 + Rb / Ra = 3.0
    IR3,
    /// 1 + Rb / Ra = 3.6
    IR4,
    /// 1 + Rb / Ra = 4.4
    IR5,
    /// 1 + Rb / Ra = 5.3
    IR6,
    /// 1 + Rb / Ra = 6.5
    IR7,
}

impl InternalResistorRatio {
    /// Converts the internal resistor ratio to a mask byte for the command.
    pub fn to_mask(&self) -> u8 {
        match self {
            InternalResistorRatio::IR0 => 0b00000000,
            InternalResistorRatio::IR1 => 0b00000001,
            InternalResistorRatio::IR2 => 0b00000010,
            InternalResistorRatio::IR3 => 0b00000011,
            InternalResistorRatio::IR4 => 0b00000100,
            InternalResistorRatio::IR5 => 0b00000101,
            InternalResistorRatio::IR6 => 0b00000110,
            InternalResistorRatio::IR7 => 0b00000111,
        }
    }
}

/// The temperature coefficient for the SSD1803A controller.
#[derive(Clone, Copy, Debug, Default)]
pub enum TempCoefficient {
    /// -0.05% / °C
    #[default]
    C0_05,
    /// -0.10% / °C
    C0_10,
    /// -0.15% / °C
    C0_15,
    /// -0.20% / °C
    C0_20,
}

impl TempCoefficient {
    /// Converts the temperature coefficient to a mask byte for the command.
    pub fn to_mask(&self) -> u8 {
        match self {
            TempCoefficient::C0_05 => 0b00000010,
            TempCoefficient::C0_10 => 0b00000100,
            TempCoefficient::C0_15 => 0b00000110,
            TempCoefficient::C0_20 => 0b00000111,
        }
    }
}

/// The ROM selection for the SSD1803A controller.
///
/// All of them are ASCII-compatible outside of control characters.
#[derive(Clone, Copy, Debug)]
pub enum Rom {
    /// ROM A (default)
    ///
    /// Contains some special symbols and latin diacritics.
    RomA,
    /// ROM B
    ///
    /// Contains Cyrillic alphabet.
    RomB,
    /// ROM C
    ///
    /// Contains some latin diacritics, a few special characters, and Japanese [katakana script](https://en.wikipedia.org/wiki/Katakana) characters.
    RomC,
}

impl Rom {
    /// Converts the ROM selection to a mask byte for the command.
    pub fn to_mask(&self) -> u8 {
        match self {
            Rom::RomA => 0b00000000,
            Rom::RomB => 0b00000100,
            Rom::RomC => 0b00001000,
        }
    }
}
