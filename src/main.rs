mod gpio;

use crate::gpio::GpioDriver;
use crate::gpio::gpiod::GpiodDriver;
use crate::gpio::lcd::hd44780::driver::{GpioHD44780Driver, HD44780Driver};
use bitvec::macros::internal::funty::Fundamental;
use dotenv::dotenv;
use gpiod::Chip;
use log::{debug, info};
use sysinfo::System;
use time::OffsetDateTime;

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
    // 8-bit data bus
    let bus_pins = [5, 6, 13, 19, 26, 16, 20, 21];
    // bus_pins.reverse();
    let mut data_bus = gpio.get_pin_bus(bus_pins)?;

    let pin_e_out = pin_e.as_output()?;
    let pin_rw_out = pin_rw.as_output()?;
    let pin_rs_out = pin_rs.as_output()?;

    let mut driver =
        GpioHD44780Driver::new_8bit(&*pin_e_out, &*pin_rw_out, &*pin_rs_out, &mut *data_bus);

    driver.init()?;

    let init = [
        0b00111010, 0b00001001, 0b00000110, 0b00011110, 0b00111001, 0b00011011, 0b01101110,
        0b01010111, 0b01110010, 0b00111000, 0b00001111,
    ];

    for i in init {
        driver.send_command(i)?;
    }

    driver.clear_display()?;

    let now = OffsetDateTime::now_local()?;
    let hms = now.to_hms();

    let str = format!("Hello @ {:02}:{:02}:{:02}", hms.0, hms.1, hms.2);
    for c in str.chars() {
        driver.send_data(c as u8)?;
    }

    driver.return_home()?;

    let (busy, addr) = driver.get_busy_flag_and_address()?;

    debug!("Busy flag: {}, Address: {:07b}", busy, addr);

    let char = driver.read_data()?;

    debug!("Read char: {}", char.as_char().unwrap());

    Ok(())
}
