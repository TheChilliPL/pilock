mod config;
mod utils;

use std::env::var;
use dotenv::dotenv;
use log::{debug, info};
use pilock_gpio::{GpioDriver};
use pilock_gpio::GpioActiveLevel::Low;
use pilock_gpio::GpioBias::PullUp;
use pilock_gpio::GpioDriveMode::OpenDrain;
use pilock_gpio::lcd::ssd1803a::driver::{BiasDivider, DoubleHeightMode, GpioSSD1803ADriver, SSD1803ADriver};
use pilock_gpio::raw::RawGpioDriver;
use pilock_gpio::keypad::{GpioKeypad, Keypad, KeypadKey};
use crate::config::Config;
use crate::utils::CollectionExt;

fn parse_pin_bus(pin_str: &str) -> eyre::Result<[usize; 4]> {
    pin_str
        .split([',', ' ', ';'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse())
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| eyre::eyre!("Invalid number of data pins"))
}

fn main() -> eyre::Result<()> {
    // Initialize environment and logger
    dotenv()?;
    pretty_env_logger::init();

    info!("PiLock starting...");

    // Get pin numbers from env
    let lcd_e_pin_no: usize = var("PILOCK_LCD_PIN_E")?.parse()?;
    let lcd_rw_pin_no: usize = var("PILOCK_LCD_PIN_RW")?.parse()?;
    let lcd_rs_pin_no: usize = var("PILOCK_LCD_PIN_RS")?.parse()?;
    let lcd_data_pin_nos: [usize; 4] = parse_pin_bus(&var("PILOCK_LCD_PINS_DATA")?)?;

    let keypad_pin_col_nos: [usize; 4] = parse_pin_bus(&var("PILOCK_KEYPAD_PINS_COLS")?)?;
    let keypad_pin_row_nos: [usize; 4] = parse_pin_bus(&var("PILOCK_KEYPAD_PINS_ROWS")?)?;

    info!("LCD @ E: {}, RW: {}, RS: {}, Data: {:?}",
        lcd_e_pin_no, lcd_rw_pin_no, lcd_rs_pin_no, lcd_data_pin_nos);
    info!("Keypad @ Cols: {:?}, Rows: {:?}", keypad_pin_col_nos, keypad_pin_row_nos);

    debug!("Initializing GPIO driver...");
    let gpio = RawGpioDriver::new_gpiomem()?;
    debug!("{:?} initialized.", gpio);

    debug!("Initializing LCD driver...");
    let mut lcd_e_pin = gpio.get_pin(lcd_e_pin_no)?;
    let lcd_e_out = lcd_e_pin.as_output()?;
    let mut lcd_rw_pin = gpio.get_pin(lcd_rw_pin_no)?;
    let lcd_rw_out = lcd_rw_pin.as_output()?;
    let mut lcd_rs_pin = gpio.get_pin(lcd_rs_pin_no)?;
    let lcd_rs_out = lcd_rs_pin.as_output()?;
    let mut lcd_data_bus = gpio.get_pin_bus(lcd_data_pin_nos)?;
    let mut lcd = GpioSSD1803ADriver::new_4bit(
        None,
        &*lcd_e_out,
        Some(&*lcd_rw_out),
        &*lcd_rs_out,
        &mut *lcd_data_bus,
    );

    lcd.init(4)?;

    lcd.clear_display()?;
    lcd.double_height_bias_dot_shift(
        DoubleHeightMode::DoubleMiddle,
        BiasDivider::default(),
        false,
    )?;
    lcd.function_set_0(
        false,
        true,
        true,
        false,
    )?;

    const STR_1: &str = "Initializing";
    const STR_2: &str = "PiLock";
    const STR_3: &str = concat!("v.", env!("CARGO_PKG_VERSION", "UNKNOWN"), "...");

    for char in STR_1.chars() {
        lcd.send_data(char as u8)?;
    }

    lcd.set_ddram_address(0x20 + 7)?;

    for char in STR_2.chars() {
        lcd.send_data(char as u8)?;
    }

    let str_3_chars = STR_3.chars().collect::<Vec<_>>();
    lcd.set_ddram_address(0x40 + 20 - str_3_chars.len() as u8)?;

    for char in str_3_chars {
        lcd.send_data(char as u8)?;
    }

    debug!("{:?} initialized.", lcd);

    debug!("Initializing keypad driver...");
    let mut keypad_col_bus = gpio.get_pin_bus(keypad_pin_col_nos)?;
    let mut keypad_row_bus = gpio.get_pin_bus(keypad_pin_row_nos)?;
    keypad_col_bus.set_drive_mode(OpenDrain)?;
    keypad_col_bus.set_active_level(Low)?;
    keypad_row_bus.set_bias(PullUp)?;
    keypad_row_bus.set_active_level(Low)?;
    let keypad_col_out = keypad_col_bus.as_output()?;
    let keypad_row_in = keypad_row_bus.as_input()?;

    let keypad = GpioKeypad::new(&*keypad_col_out, &*keypad_row_in);

    debug!("{:?} initialized.", keypad);

    debug!("Trying to load config...");
    // let config = config::Config::try_load();
    let config = if let Some(config) = Config::try_load() {
        info!("Config loaded.");
        config
    } else {
        info!("Config not found. Using default");
        let config = Config::default();
        config.save()?;
        info!("Default config saved.");
        config
    };

    debug!("Password is {:?}.", config.password);

    info!("PiLock initialized.");

    std::thread::sleep(std::time::Duration::from_secs(1));

    info!("Starting main loop...");

    loop {
        let pressed = keypad.read()?;
        // debug!("Pressed: {:?}", pressed);

        let key = pressed.try_get_single().cloned().ok();

        if let Some(key) = key {
            debug!("Key pressed: {:?}", key);

            if key == KeypadKey::KeyHash {
                return Ok(());
            }
        }
        
        // Sleep for 1/20th of a second
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Ok(())
}
