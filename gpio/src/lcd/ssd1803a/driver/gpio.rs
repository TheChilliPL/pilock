use std::thread::sleep;
use std::time::Duration;
use log::trace;
use crate::{GpioBus, GpioError, GpioOutput, GpioResult};
use crate::lcd::hd44780::driver::GpioHD44780Bus;
use crate::lcd::ssd1803a::driver::{BiasDivider, DoubleHeightMode, InternalOscFrequency, InternalResistorRatio, SSD1803ADriver};

/// GpioSSD1803A driver for SSD1803A LCD controller using GPIO pins.
/// 
/// The E pin is pulsed for 1 us and then the display gets 1 ms to process the command, which should be more than enough for every command.
/// It made the code simpler and more reliable, as the timing is not critical for the SSD1803A controller, and in this case we wait the same
/// duration for every command.
#[derive(Debug)]
pub struct GpioSSD1803ADriver<'a> {
    pin_reset: Option<&'a dyn GpioOutput>,
    pin_e: &'a dyn GpioOutput,
    pin_rw: Option<&'a dyn GpioOutput>,
    pin_rs: &'a dyn GpioOutput,
    data_bus: GpioHD44780Bus<'a>,

    lines: u8,
    re_state: bool,
    is_state: bool,
    blink_state: bool,
    invert_state: bool,
    double_height_state: bool,
}

impl<'a> GpioSSD1803ADriver<'a> {
    /// Creates a new GpioSSD1803ADriver instance for SSD1803A LCD controller, using 4-bit data bus.
    ///
    /// # Parameters
    ///
    /// - `pin_reset`: Optional reset pin, used to reset the display, as recommended by the SSD1803A datasheet.
    ///   The display might not initialize properly without it. The reset pin **must be set to active low**.
    /// - `pin_e`: Enable output pin.
    /// - `pin_rw`: Optional read/write output pin. If not provided, the driver will only support write operations
    ///   (R/W pin of the display must be connected to GND, signifying write mode).
    /// - `pin_rs`: Register select output pin.
    /// - `data_bus`: The data bus used for communication with the SSD1803A controller.
    pub fn new_4bit(
        pin_reset: Option<&'a dyn GpioOutput>,
        pin_e: &'a dyn GpioOutput,
        pin_rw: Option<&'a dyn GpioOutput>,
        pin_rs: &'a dyn GpioOutput,
        data_bus: &'a mut dyn GpioBus<4>,
    ) -> Self {
        GpioSSD1803ADriver {
            pin_reset,
            pin_e,
            pin_rw,
            pin_rs,
            data_bus: GpioHD44780Bus::Bus4Bit(data_bus),
            lines: 1,
            re_state: false,
            is_state: false,
            blink_state: false,
            invert_state: false,
            double_height_state: false,
        }
    }

    /// Creates a new GpioSSD1803ADriver instance for SSD1803A LCD controller, using 8-bit data bus.
    ///
    /// # Parameters
    ///
    /// - `pin_reset`: Optional reset pin, used to reset the display, as recommended by the SSD1803A datasheet.
    ///   The display might not initialize properly without it. The reset pin **must be set to active low**.
    /// - `pin_e`: Enable output pin.
    /// - `pin_rw`: Optional read/write output pin. If not provided, the driver will only support write operations
    ///   (R/W pin of the display must be connected to GND, signifying write mode).
    /// - `pin_rs`: Register select output pin.
    /// - `data_bus`: The data bus used for communication with the SSD1803A controller.
    pub fn new_8bit(
        pin_reset: Option<&'a dyn GpioOutput>,
        pin_e: &'a dyn GpioOutput,
        pin_rw: Option<&'a dyn GpioOutput>,
        pin_rs: &'a dyn GpioOutput,
        data_bus: &'a mut dyn GpioBus<8>,
    ) -> Self {
        GpioSSD1803ADriver {
            pin_reset,
            pin_e,
            pin_rw,
            pin_rs,
            data_bus: GpioHD44780Bus::Bus8Bit(data_bus),
            lines: 1,
            re_state: false,
            is_state: false,
            blink_state: false,
            invert_state: false,
            double_height_state: false,
        }
    }

    fn pulse_pin(pin: &dyn GpioOutput) -> GpioResult<()> {
        pin.write(true)?;
        sleep(Duration::from_micros(1));
        pin.write(false)?;
        sleep(Duration::from_millis(1));
        Ok(())
    }

    fn send(&mut self, data: u8, rs: bool) -> GpioResult<()> {
        trace!("Sending data: {:08b}, RS: {}", data, rs);

        // Set RS pin
        self.pin_rs.write(rs)?;

        // Set RW pin to write
        if let Some(rw) = self.pin_rw {
            rw.write(false)?;
        }

        // Write data to the data bus
        match &mut self.data_bus {
            GpioHD44780Bus::Bus8Bit(bus) => {
                let bus = bus.as_output()?;
                bus.write_byte(data)?;
                Self::pulse_pin(self.pin_e)?;
            }
            GpioHD44780Bus::Bus4Bit(bus) => {
                let high_nibble = (data >> 4) & 0x0F;
                let low_nibble = data & 0x0F;
                {
                    let bus = bus.as_output()?;
                    trace!("Writing HN: {:04b}", high_nibble);
                    bus.write_nibble(high_nibble)?;
                    Self::pulse_pin(self.pin_e)?;
                }
                if let GpioHD44780Bus::Bus4Bit(bus) = &mut self.data_bus {
                    let bus = bus.as_output()?;
                    trace!("Writing LN: {:04b}", low_nibble);
                    bus.write_nibble(low_nibble)?;
                }
                Self::pulse_pin(self.pin_e)?;
            }
        }

        Ok(())
    }

    fn read(&mut self, rs: bool) -> GpioResult<u8> {
        if self.pin_rw.is_none() {
            return Err(GpioError::NotSupported);
        }

        // Read data from the data bus
        let data = match &mut self.data_bus {
            GpioHD44780Bus::Bus8Bit(bus) => {
                let input = bus.as_input()?;

                // Set RS pin
                self.pin_rs.write(rs)?;

                // Set RW pin to read
                self.pin_rw.unwrap().write(true)?;
                sleep(Duration::from_micros(1));

                // Set E pin to high
                self.pin_e.write(true)?;
                sleep(Duration::from_micros(1));

                let data = input.read_byte()?;

                self.pin_e.write(false)?;
                sleep(Duration::from_micros(1));

                data
            }
            GpioHD44780Bus::Bus4Bit(bus) => {
                let input = bus.as_input()?;

                // Set RS pin
                self.pin_rs.write(rs)?;

                // Set RW pin to read
                self.pin_rw.unwrap().write(true)?;
                sleep(Duration::from_micros(1));

                // Set E pin to high
                self.pin_e.write(true)?;
                sleep(Duration::from_micros(1));

                let high_nibble = input.read_nibble()?;

                self.pin_e.write(false)?;
                sleep(Duration::from_micros(1));

                self.pin_e.write(true)?;
                sleep(Duration::from_micros(1));

                let low_nibble = input.read_nibble()?;

                self.pin_e.write(false)?;
                sleep(Duration::from_micros(1));

                (high_nibble << 4) | low_nibble
            }
        };

        // Set RW pin back to write
        self.pin_rw.unwrap().write(false)?;

        trace!("Read data: {:08b}, RS: {}", data, rs);

        Ok(data)
    }
}

impl SSD1803ADriver for GpioSSD1803ADriver<'_> {
    /// Initializes the SSD1803A display with the specified number of lines.
    /// 
    /// First, it pulses the reset pin if provided, which is recommended by the SSD1803A datasheet.
    ///
    /// It starts by doing a synchronization sequence. It sends `0011`(`1000`) bit three times
    /// to ensure that the display is set to 8-bit mode, no matter what the current mode was.
    /// If 4-bit, then sends `0010` to set it to 4-bit mode.
    ///
    /// Then, it commences with the recommended DOGM204-A initialization sequence:
    /// - sets line number with the extended function set command,
    /// - sets appropriate¹ entry mode for the display to be oriented properly,
    /// - sets the bias divider to default,
    /// - sets the internal oscillator frequency to default,
    /// - turns on the divider circuit and sets appropriate¹ internal resistor ratio,
    /// - turns the booster on and sets the contrast to default,
    /// - finishes setting the line number with the standard function set command,
    /// - turns the display on,
    /// - clears the display.
    ///
    /// After this intricate sequence, the display should be fully ready for use.
    fn init(&mut self, lines: u8) -> GpioResult<()> {
        const DEFAULT_CONTRAST: u8 = 0b11010;

        if lines < 1 || lines > 4 {
            return Err(GpioError::InvalidArgument);
        }

        self.lines = lines;
        
        // Pulse reset pin if provided
        if let Some(pin_reset) = self.pin_reset {
            pin_reset.write(true)?;
            sleep(Duration::from_millis(10));
            pin_reset.write(false)?;
            sleep(Duration::from_millis(10));
        }

        // Synchronize
        match self.data_bus {
            GpioHD44780Bus::Bus8Bit(_) => {
                self.send(0b00111000, false)?;
                self.send(0b00111000, false)?;
                self.send(0b00111000, false)?;
            }
            GpioHD44780Bus::Bus4Bit(_) => {
                self.send(0b00110011, false)?;
                self.send(0b00110010, false)?;
            }
        }
        // RE = 1 --- should happen automatically
        // self.function_set_1(
        //     self.data_bus.is_8bit(),
        //     lines == 2 || lines == 4,
        //     false,
        //     false,
        // )?;
        // 4-line display
        self.ext_function_set(false, false, lines >= 3)?;
        // Bottom view
        self.set_entry_mode_ex(false, true)?;
        // Bias setting
        self.double_height_bias_dot_shift(
            DoubleHeightMode::default(),
            BiasDivider::default(),
            false,
        )?;
        // RE = 0 && IS = 1 --- should happen automatically
        // self.function_set_0(
        //     self.data_bus.is_8bit(),
        //     lines == 2 || lines == 4,
        //     false,
        //     true,
        // )?;
        // Internal OSC
        self.internal_osc_frequency(
            BiasDivider::default(),
            InternalOscFrequency::default(),
        )?;
        // Follower control: divider on and
        self.follower_control(
            true,
            InternalResistorRatio::IR6,
        )?;
        // Power control
        self.icon_booster_contrast(
            false,
            true,
            DEFAULT_CONTRAST,
        )?;
        // Contrast set
        self.contrast_set(DEFAULT_CONTRAST)?;
        // IS = 0 --- DOESN'T happen automatically
        self.function_set_0(
            self.data_bus.is_8bit(),
            lines == 2 || lines == 4,
            false,
            false,
        )?;
        // Set display on
        self.set_display_control(true, false, false)?;
        // Clear display
        self.clear_display()?;

        Ok(())
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
        self.lines = match self.lines {
            1 | 2 => if lines_2_or_4 { 2 } else { 1 },
            3 | 4 => if lines_2_or_4 { 4 } else { 3 },
            _ => return Err(GpioError::InvalidArgument),
        };
        self.double_height_state = double_height_enabled;
        self.re_state = false;
        self.is_state = set_is;
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
        self.lines = match self.lines {
            1 | 2 => if lines_2_or_4 { 2 } else { 1 },
            3 | 4 => if lines_2_or_4 { 4 } else { 3 },
            _ => return Err(GpioError::InvalidArgument),
        };
        self.blink_state = char_blink;
        self.invert_state = invert;
        self.re_state = true;
        self.send_command(command, None, None)
    }

    /// Sends the specific command by setting RS to 0, the data pins to the command value,
    /// and pulsing the E pin.
    /// 
    /// If the IS or RE state needs to be changed, it will call the appropriate function set commands first.
    fn send_command(&mut self, data: u8, is: Option<bool>, re: Option<bool>) -> GpioResult<()> {
        let needs_is_set = is.is_some_and(|is| is != self.is_state);

        if needs_is_set {
            self.function_set_0(
                self.data_bus.is_8bit(),
                self.lines == 2 || self.lines == 4,
                false,
                is.unwrap(),
            )?;
        }

        let needs_re_set = re.is_some_and(|re| re != self.re_state);

        if needs_re_set {
            match re.unwrap() {
                true => self.function_set_1(
                    self.data_bus.is_8bit(),
                    self.lines == 2 || self.lines == 4,
                    self.blink_state,
                    self.invert_state,
                )?,
                false => self.function_set_0(
                    self.data_bus.is_8bit(),
                    self.lines == 2 || self.lines == 4,
                    self.double_height_state,
                    self.is_state,
                )?,
            }
        }

        self.send(data, false)?;

        Ok(())
    }

    /// Sends the specific data by setting RS to 1, the data pins to the data value,
    /// and pulsing the E pin.
    fn send_data(&mut self, data: u8) -> GpioResult<()> {
        self.send(data, true)
    }

    /// Reads a command from the display by setting RS to 0, the R/W pin to read,
    /// and pulsing the E pin. It reads the data bus *during* the E pin pulse.
    fn read_command(&mut self) -> GpioResult<u8> {
        self.read(false)
    }

    /// Reads data from the display by setting RS to 1, the R/W pin to read,
    /// and pulsing the E pin. It reads the data bus *during* the E pin pulse.
    fn read_data(&mut self) -> GpioResult<u8> {
        self.read(true)
    }
}
