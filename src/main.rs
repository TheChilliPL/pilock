mod gpio;

use crate::gpio::GpioDriver;
use crate::gpio::gpiod::GpiodDriver;
use crate::gpio::lcd::hd44780::driver::{GpioHD44780Driver, HD44780Driver};
use dotenv::dotenv;
use gpiod::Chip;
use log::info;
use std::thread::sleep;
use std::time::Duration;
use sysinfo::System;
use time::OffsetDateTime;
use crate::gpio::lcd::ssd1803a::driver::{GpioSSD1803ADriver, SSD1803ADriver};
use crate::gpio::lcd::ssd1803a::driver::DoubleHeightMode::DoubleBoth;
use crate::gpio::pwm::{PwmDriver, PwmExtension, SysfsPwmDriver};

fn main() -> eyre::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    const UNKNOWN_STR: &str = "???";

    info!(
        "Hello, {}!",
        System::name().as_deref().unwrap_or(UNKNOWN_STR)
    );
    info!(
        "System ver {} kernel ver {}",
        System::long_os_version().as_deref().unwrap_or(UNKNOWN_STR),
        System::kernel_version().as_deref().unwrap_or(UNKNOWN_STR),
    );
    info!(
        "Hostname {}",
        System::host_name().as_deref().unwrap_or(UNKNOWN_STR)
    );
    info!("Architecture {}", System::cpu_arch());

    let gpio = GpiodDriver::new(Chip::new("/dev/gpiochip0")?);

    let mut pin_e = gpio.get_pin(17)?;
    let mut pin_rw = gpio.get_pin(27)?;
    let mut pin_rs = gpio.get_pin(22)?;
    // 8-bit data bus - D0 D1 ... D7
    let bus_pins = [5, 6, 13, 19, 26, 16, 20, 21];
    // let dead_pins = [5, 6, 13, 19];
    // let mut dead_bus = gpio.get_pin_bus(dead_pins)?;
    // dead_bus.as_output()?.write_nibble(0b1111)?;
    // let bus_pins = [26, 16, 20, 21];
    // bus_pins.reverse();

    let pin_e_out = pin_e.as_output()?;
    let pin_rw_out = pin_rw.as_output()?;
    let pin_rs_out = pin_rs.as_output()?;

    // {
    //     let mut data_bus = gpio.get_pin_bus(bus_pins)?;
    //
    //     let mut driver = GpioHD44780Driver::new_8bit(
    //         &*pin_e_out,
    //         Some(&*pin_rw_out),
    //         &*pin_rs_out,
    //         &mut *data_bus,
    //     );
    //
    //     driver.init(true, false)?;
    //
    //     let init = [
    //         0b00111010, 0b00001001, 0b00000110, 0b00011110, 0b00111001, 0b00011011, 0b01101110,
    //         0b01010111, 0b01110010, 0b00111000, 0b00001111,
    //     ];
    //
    //     for i in init {
    //         driver.send_command(i)?;
    //     }
    //
    //     driver.clear_display()?;
    //
    //     let now = OffsetDateTime::now_local()?;
    //     let hms = now.to_hms();
    //
    //     let str = format!("Hi PiLock {:02}:{:02}:{:02}", hms.0, hms.1, hms.2);
    //     for c in str.chars() {
    //         driver.send_data(c as u8)?;
    //     }
    //
    //     driver.return_home()?;
    //
    //     driver.set_ddram_address(0x20)?;
    //
    //     let str = System::kernel_version()
    //         .as_deref()
    //         .unwrap_or(UNKNOWN_STR)
    //         .to_string();
    //     for c in str.chars() {
    //         driver.send_data(c as u8)?;
    //     }
    //
    //     driver.set_ddram_address(0x40)?;
    //
    //     let str = System::cpu_arch();
    //     for c in str.chars() {
    //         driver.send_data(c as u8)?;
    //     }
    //
    //     driver.set_ddram_address(0x60)?;
    //
    //     for c in "8-bit interface works".chars() {
    //         driver.send_data(c as u8)?;
    //     }
    // }
    //
    // sleep(Duration::from_secs(2));

    let pwm = SysfsPwmDriver::get_chip(0)?;
    let mut pin_pwm = pwm.get_pin(0)?;
    pin_pwm.set_period(Duration::from_secs(1))?;
    pin_pwm.set_duty(Duration::from_millis(500))?;
    pin_pwm.enable()?;

    {
        let dead_pins = [5, 6, 13, 19];
        let bus_pins = [26, 16, 20, 21];

        let mut dead_bus = gpio.get_pin_bus(dead_pins)?;
        let mut data_bus = gpio.get_pin_bus(bus_pins)?;

        dead_bus.as_output()?.write_nibble(0b0000)?;

        let mut driver = GpioSSD1803ADriver::new_4bit(
            None,
            &*pin_e_out,
            Some(&*pin_rw_out),
            &*pin_rs_out,
            &mut *data_bus,
        );

        driver.init(4)?;

        let str = "Hi PiLock 4-bit";

        for c in str.chars() {
            driver.send_data(c as u8)?;
        }

        // driver.send_command(0b00101010)?; // RE = 1
        // driver.send_command(0b00011011)?; // Double height bottom
        // driver.send_command(0b00101100)?; // RE = 0, double height on
        driver.double_height_bias_dot_shift(DoubleBoth, Default::default(), false)?;
        driver.function_set_0(false, true, true, false)?;

        
        loop {
            let time = OffsetDateTime::now_local()?;
            let (h, m, s) = time.to_hms();
            
            driver.set_ddram_address(0x26)?;
            
            let str = format!("{:02}:{:02}:{:02}", h, m, s);
            for c in str.chars() {
                driver.send_data(c as u8)?;
            }
        }
    }

    Ok(())
}
