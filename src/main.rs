mod gpio;

use std::ops::Add;
use crate::gpio::GpioDriver;
use crate::gpio::gpiod::GpiodDriver;
use crate::gpio::lcd::hd44780::driver::{GpioHD44780Driver, HD44780Driver};
use dotenv::dotenv;
use gpiod::Chip;
use log::{debug, info};
use std::thread::sleep;
use std::time::Duration;
use sysinfo::System;
use time::OffsetDateTime;
use crate::gpio::lcd::ssd1803a::driver::{GpioSSD1803ADriver, SSD1803ADriver};
use crate::gpio::lcd::ssd1803a::driver::DoubleHeightMode::DoubleBottom;
use crate::gpio::pwm::{PwmDriver, PwmExtension, RawPwmDriver, SysfsPwmDriver};
use crate::gpio::raw::RawGpioDriver;

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

    let gpio = RawGpioDriver::new_gpiomem()?;
    // let gpio = GpiodDriver::new(Chip::new("/dev/gpiochip0")?);

    let mut pin_e = gpio.get_pin(17)?;
    let mut pin_rw = gpio.get_pin(27)?;
    let mut pin_rs = gpio.get_pin(22)?;

    // 4-bit data bus - D0 D1 ... D3
    let bus_pins = [26, 16, 20, 21];

    let pin_e_out = pin_e.as_output()?;
    let pin_rw_out = pin_rw.as_output()?;
    let pin_rs_out = pin_rs.as_output()?;

    // let pwm = SysfsPwmDriver::get_chip(0)?;
    // let mut pin_pwm = pwm.get_pin(0)?;
    // pin_pwm.set_period(Duration::from_secs(1))?;
    // pin_pwm.set_duty(Duration::from_millis(500))?;
    // pin_pwm.enable()?;

    let pwm = RawPwmDriver::new_mem(0)?;

    {
        let mut data_bus = gpio.get_pin_bus(bus_pins)?;

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

        driver.set_ddram_address(0x20)?;

        let str = System::cpu_arch();

        for c in str.chars() {
            driver.send_data(c as u8)?
        }

        driver.double_height_bias_dot_shift(DoubleBottom, Default::default(), false)?;
        driver.function_set_0(false, true, true, false)?;

        loop {
            let time = OffsetDateTime::now_local()?;
            let (h, m, s) = time.to_hms();
            
            driver.set_ddram_address(0x36)?;
            
            let str = format!("{:02}:{:02}:{:02}", h, m, s);
            for c in str.chars() {
                driver.send_data(c as u8)?;
            }

            // Delay until the start of the next second
            let next_second = time.clone()
                .add(Duration::from_secs(1))
                .replace_nanosecond(0)?;
            sleep(Duration::try_from(next_second - time)?);
        }
    }

    Ok(())
}
