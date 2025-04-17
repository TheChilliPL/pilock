mod gpio;

use std::fmt::Debug;
use crate::gpio::{GpioError, GpioResult};
use crate::gpio::lcd::hd44780::driver::{CursorDirection, HD44780Driver};
pub use gpio::*;

pub trait SSD1803ADriver: Debug {
    /// Initializes the SSD1803A controller with the default settings.
    fn init(&mut self, lines: u8) -> GpioResult<()>;

    fn clear_display(&mut self) -> GpioResult<()> {
        self.send_command(0b00000001, None, None)
    }

    fn return_home(&mut self) -> GpioResult<()> {
        self.send_command(0b00000010, None, Some(false))
    }

    fn power_down_mode(&mut self, power_down: bool) -> GpioResult<()> {
        let mut command = 0b00000010;
        if power_down {
            command |= 0b00000001;
        }
        self.send_command(command, None, Some(true))
    }

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

    fn set_cgram_address(&mut self, address: u8) -> GpioResult<()> {
        if address > 0b00111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b01000000 | address;
        self.send_command(command, Some(false), Some(false))
    }

    fn set_segram_address(&mut self, address: u8) -> GpioResult<()> {
        if address > 0b00001111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b01000000 | address;
        self.send_command(command, Some(true), Some(false))
    }

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

    fn contrast_set(&mut self, contrast: u8) -> GpioResult<()> {
        if contrast > 0b00111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b01110000 | (contrast & 0b1111);
        self.send_command(command, Some(true), Some(false))
    }

    fn set_ddram_address(&mut self, address: u8) -> GpioResult<()> {
        if address > 0b01111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b10000000 | address;
        self.send_command(command, None, Some(false))
    }

    fn set_scroll_quantity(&mut self, quantity: u8) -> GpioResult<()> {
        if quantity > 0b00111111 {
            return Err(GpioError::InvalidArgument);
        }
        let command = 0b10000000 | quantity;
        self.send_command(command, None, Some(true))
    }

    fn get_busy_flag_and_address(&mut self) -> GpioResult<(bool, u8)> {
        let command = self.read_command()?;
        let busy_flag = command & 0b10000000 != 0;
        let address = command & 0b01111111;
        Ok((busy_flag, address))
    }

    fn temp_coefficient_control(
        &mut self,
        temp_coefficient: TempCoefficient,
    ) -> GpioResult<()> {
        self.send_command(0b01110110, None, Some(true))?;
        self.send_data(temp_coefficient.to_mask())
    }

    fn rom_selection(
        &mut self,
        rom: Rom,
    ) -> GpioResult<()> {
        self.send_command(0b01110010, None, Some(true))?;
        self.send_data(rom.to_mask())
    }

    /// Sends a command to the SSD1803A controller.
    ///
    /// Before sending the raw command, ensures IS and RE bits are set correctly,
    /// and sets them otherwise.
    fn send_command(&mut self, data: u8, is: Option<bool>, re: Option<bool>) -> GpioResult<()>;

    fn send_data(&mut self, data: u8) -> GpioResult<()>;

    fn read_command(&mut self) -> GpioResult<u8>;

    fn read_data(&mut self) -> GpioResult<u8>;
}

// Compatibility layer for HD44780
impl HD44780Driver for dyn SSD1803ADriver {
    fn init(&mut self, multiline: bool, alt_font: bool) -> GpioResult<()> {
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

#[derive(Clone, Copy, Debug, Default)]
pub enum DoubleHeightMode {
    DoubleBottom,
    DoubleMiddle,
    DoubleBoth,
    #[default]
    DoubleTop,
}

impl DoubleHeightMode {
    pub fn to_mask(&self) -> u8 {
        match self {
            DoubleHeightMode::DoubleBottom => 0b00000000,
            DoubleHeightMode::DoubleMiddle => 0b00000100,
            DoubleHeightMode::DoubleBoth =>   0b00001000,
            DoubleHeightMode::DoubleTop =>    0b00001100,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum BiasDivider {
    Bias1_5,
    Bias1_4,
    Bias1_7,
    #[default]
    Bias1_6,
}

impl BiasDivider {
    pub fn bs1(&self) -> bool {
        match self {
            BiasDivider::Bias1_5 => false,
            BiasDivider::Bias1_4 => false,
            BiasDivider::Bias1_7 => true,
            BiasDivider::Bias1_6 => true,
        }
    }

    pub fn bs0(&self) -> bool {
        match self {
            BiasDivider::Bias1_5 => false,
            BiasDivider::Bias1_4 => true,
            BiasDivider::Bias1_7 => false,
            BiasDivider::Bias1_6 => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum InternalOscFrequency {
    Freq420kHz,
    Freq460kHz,
    Freq500kHz,
    #[default] Freq540kHz,
    Freq580kHz,
    Freq620kHz,
    Freq640kHz,
    Freq680kHz,
}

impl InternalOscFrequency {
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

#[derive(Clone, Copy, Debug, Default)]
enum InternalResistorRatio {
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
    pub fn to_mask(&self) -> u8 {
        match self {
            TempCoefficient::C0_05 => 0b00000010,
            TempCoefficient::C0_10 => 0b00000100,
            TempCoefficient::C0_15 => 0b00000110,
            TempCoefficient::C0_20 => 0b00000111,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rom {
    RomA,
    RomB,
    RomC,
}

impl Rom {
    pub fn to_mask(&self) -> u8 {
        match self {
            Rom::RomA => 0b00000000,
            Rom::RomB => 0b00000100,
            Rom::RomC => 0b00001000,
        }
    }
}
