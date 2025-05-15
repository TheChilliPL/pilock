use crate::lcd::hd44780::driver::{CursorDirection, HD44780Driver};
use crate::{GpioBus, GpioError, GpioOutput, GpioResult};
use log::trace;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug)]
pub enum GpioHD44780Bus<'a> {
    Bus8Bit(&'a mut dyn GpioBus<8>),
    Bus4Bit(&'a mut dyn GpioBus<4>),
}

impl GpioHD44780Bus<'_> {
    pub fn is_8bit(&self) -> bool {
        matches!(self, GpioHD44780Bus::Bus8Bit(_))
    }

    pub fn is_4bit(&self) -> bool {
        matches!(self, GpioHD44780Bus::Bus4Bit(_))
    }
}

#[derive(Debug)]
pub struct GpioHD44780Driver<'a> {
    pin_e: &'a dyn GpioOutput,
    pin_rw: Option<&'a dyn GpioOutput>,
    pin_rs: &'a dyn GpioOutput,
    data_bus: GpioHD44780Bus<'a>,
}

impl<'a> GpioHD44780Driver<'a> {
    pub fn new_4bit(
        pin_e: &'a dyn GpioOutput,
        pin_rw: Option<&'a dyn GpioOutput>,
        pin_rs: &'a dyn GpioOutput,
        data_bus: &'a mut dyn GpioBus<4>,
    ) -> Self {
        GpioHD44780Driver {
            pin_e,
            pin_rw,
            pin_rs,
            data_bus: GpioHD44780Bus::Bus4Bit(data_bus),
        }
    }

    pub fn new_8bit(
        pin_e: &'a dyn GpioOutput,
        pin_rw: Option<&'a dyn GpioOutput>,
        pin_rs: &'a dyn GpioOutput,
        data_bus: &'a mut dyn GpioBus<8>,
    ) -> Self {
        GpioHD44780Driver {
            pin_e,
            pin_rw,
            pin_rs,
            data_bus: GpioHD44780Bus::Bus8Bit(data_bus),
        }
    }

    fn pulse_e(pin: &dyn GpioOutput) -> GpioResult<()> {
        // Set E pin to high
        pin.write(true)?;
        sleep(Duration::from_micros(1));
        // Set E pin to low
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
                Self::pulse_e(self.pin_e)?;
            }
            GpioHD44780Bus::Bus4Bit(bus) => {
                let high_nibble = (data >> 4) & 0x0F;
                let low_nibble = data & 0x0F;
                {
                    let bus = bus.as_output()?;
                    trace!("Writing HN: {:04b}", high_nibble);
                    bus.write_nibble(high_nibble)?;
                    Self::pulse_e(self.pin_e)?;
                }
                if let GpioHD44780Bus::Bus4Bit(bus) = &mut self.data_bus {
                    let bus = bus.as_output()?;
                    trace!("Writing LN: {:04b}", low_nibble);
                    bus.write_nibble(low_nibble)?;
                }
                Self::pulse_e(self.pin_e)?;
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

impl HD44780Driver for GpioHD44780Driver<'_> {
    fn init(&mut self, multiline: bool, alt_font: bool) -> GpioResult<()> {
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
        self.function_set(self.data_bus.is_8bit(), multiline, alt_font)?;
        self.clear_display()?;
        self.set_display_control(true, false, false)?;
        self.set_entry_mode(CursorDirection::Right, false)?;
        Ok(())
    }

    fn send_command(&mut self, command: u8) -> GpioResult<()> {
        self.send(command, false)
    }

    fn send_data(&mut self, data: u8) -> GpioResult<()> {
        self.send(data, true)
    }

    fn read_command(&mut self) -> GpioResult<u8> {
        self.read(false)
    }

    fn read_data(&mut self) -> GpioResult<u8> {
        self.read(true)
    }
}
