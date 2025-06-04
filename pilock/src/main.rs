//! # PiLock main application
//! This is the main application crate for PiLock, a Raspberry Pi-based electronic door lock made for
//! Lodz University of Technology's Embedded Systems course.
//!
//! The entirety (100%) of the project was made by 247617 Patryk Anuszczyk.
//!
//! This project is written in Rust, with the documentation written in Rustdoc format.
//!
//! # Features
//!
//! - **GPIO** --- provides access to GPIO pins by writing to the GPIO registers
//!   via `/dev/gpiomem` or `/dev/mem` with a specific offset. The implementation
//!   and documentation can be found at [pilock_gpio::raw::RawGpioDriver].
//! - **Debouncing** --- debounces input GPIO pins using a timed debounce, wrapping
//!   any other GPIO input implementation. The implementation and documentation
//!   can be found at [pilock_gpio::debounce::TimedDebounce].
//! - **PWM** --- provides access to PWM pins by writing to the PWM registers.
//!   The implementation and documentation can be found at [pilock_gpio::pwm::RawPwmDriver].
//! - **SSD1803A LCD** --- provides access to the SSD1803A LCD driver, mostly
//!   compatible with the HD44780 LCD driver. The implementation and documentation
//!   can be found at [pilock_gpio::lcd::ssd1803a::driver::GpioSSD1803ADriver].
//! - **Buzzer** --- the buzzer is controlled using a PWM pin. The melodies are made
//!   using the [MusicalNote] enum and the [melody!] macro **in the main project**.
//! - **Rotary Encoder** --- allows to use a rotary encoder to control the project.
//!   It requires two GPIO pins for the encoder, and requires 2 pulses to correctly
//!   detect the rotation, as it works with the specific encoder used. The implementation
//!   and documentation can be found at [pilock_gpio::rotenc::RotEnc].
//!
//! As you can see, all GPIO-related functionality is implemented in the
//! [pilock_gpio] crate. Everything is documented there. You can find everything
//! you need by clicking on all the links in the documentation, but notice that
//! clicking on any type, method, etc. provided by a library (time-related for example),
//! even by the standard library, will take you to the documentation of that
//! crate instead of the documentation of PiLock.
//!
//! # Building
//!
//! To build the project, you need to have Rust installed. The easiest way to cross-compile
//! the project for the Raspberry Pi is to use the `cross` tool.
//!
//! To build the project, run:
//! ```bash
//! cross build --target=aarch64-unknown-linux-gnu
//! ```
//! and then copy the resulting binary from `target/aarch64-unknown-linux-gnu/debug/pilock`
//! to the Raspberry Pi using `scp` or any other method.
//!
//! Ensure the binary has the executable permission so that it can be run.
//!
//! # Running
//!
//! To run PiLock after transferring the binary, simply do
//! ```bash
//! sudo ./pilock
//! ```
//!
//! The `sudo` is required due to the need for raw memory access for PWM and the clock.
//! The GPIO driver itself could work without root permissions through `/dev/gpiomem` if the
//! project were modified.
//!
//! # Configuration
//!
//! ## Environment Variables
//!
//! The project uses environment variables to configure the GPIO pins and other settings.
//!
//! The following environment variables are used:
//! - `RUST_LOG` --- sets the log level for the application. Set to `DEBUG` to see most useful info.
//! - `PILOCK_LCD_PIN_E`, `PILOCK_LCD_PIN_RW`, `PILOCK_LCD_PIN_RS` --- control pins for the display.
//! - `PILOCK_LCD_PINS_DATA` --- data pins for the display, in 4-bit mode, comma-separated (D4,D5,D6,D7).
//!
//! ## Configuration File
//!
//! Upon first run, PiLock will create a configuration file in the current directory named `pilock.json`.
//!
//! This file contains the following fields:
//! - `password` --- the password for the lock, an array of digits, each of which is a string.
//! - `unlock_seconds` --- the number of seconds to keep the lock unlocked after entering the password.
//! - `contrast` --- the contrast of the LCD, a value between 0 and 63.
//!
//! # User Guide
//!
//! Upon starting the application, and after waiting for the initialization sequence
//! to end, the lock will initialize to the locked mode.
//!
//! The lock can be unlocked by entering the correct password using the rotary encoder.
//!
//! Rotating the encoder will change the last visible digit, and pressing the button
//! will confirm the digit and move to the next one. After inputting all digits,
//! the lock will check if the password is correct.
//!
//! If the password is correct, the lock will unlock for the configured number of seconds,
//! while playing a happy melody with the buzzer.
//!
//! If the password is incorrect, the lock will remain locked and play a sad melody.
//!
//! As a bonus, one can input the secret `0915` code to play Megalovania from Undertale ;)
//! This is meant to show off the musical capabilities of the buzzer.
#![deny(missing_docs)]

mod config;
mod utils;
mod app;
mod notes;

use crate::notes::MusicalNote;
use std::env::var;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};
use dotenv::dotenv;
use log::{debug, info};
use pilock_gpio::{GpioActiveLevel, GpioBias, GpioDriver};
use pilock_gpio::clock::{ClockDriver, MashMode};
use pilock_gpio::clock::raw::RawClockDriver;
use pilock_gpio::debounce::TimedDebounce;
use pilock_gpio::GpioActiveLevel::Low;
use pilock_gpio::GpioBias::PullUp;
use pilock_gpio::GpioDriveMode::OpenDrain;
use pilock_gpio::lcd::ssd1803a::driver::{BiasDivider, DoubleHeightMode, GpioSSD1803ADriver, SSD1803ADriver};
use pilock_gpio::raw::RawGpioDriver;
use pilock_gpio::keypad::{GpioKeypad, Keypad, KeypadKey};
use pilock_gpio::pwm::{PwmDriver, PwmExtension, RawPwmDriver};
use pilock_gpio::rotenc::RotEnc;
use pilock_music_proc_macro::note;
use crate::app::App;
use crate::config::Config;
use crate::utils::{CollectionExt, DisplayExt};

/// Parses a pin bus string (comma-separated) into an array of 4 pin numbers.
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

/// The main function of the PiLock application. Initializes all the drivers, configures the LCD,
/// and then enters the main loop in which [App::update] is called repeatedly.
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
    
    lcd.print("Initializing")?;

    lcd.set_cursor(1, 7)?;
    
    lcd.print("PiLock")?;

    const LAST_LINE: &'static str = concat!("v.", env!("CARGO_PKG_VERSION", "UNKNOWN"), "...");
    
    lcd.set_cursor(2, 20 - LAST_LINE.len())?;

    lcd.print(LAST_LINE)?;

    debug!("{:?} initialized.", lcd);

    // debug!("Initializing keypad driver...");
    // let mut keypad_col_bus = gpio.get_pin_bus(keypad_pin_col_nos)?;
    // let mut keypad_row_bus = gpio.get_pin_bus(keypad_pin_row_nos)?;
    // keypad_col_bus.set_drive_mode(OpenDrain)?;
    // keypad_col_bus.set_active_level(Low)?;
    // keypad_row_bus.set_bias(PullUp)?;
    // keypad_row_bus.set_active_level(Low)?;
    // let keypad_col_out = keypad_col_bus.as_output()?;
    // let keypad_row_in = keypad_row_bus.as_input()?;
    //
    // let mut keypad = GpioKeypad::new(&*keypad_col_out, &*keypad_row_in);
    //
    // debug!("{:?} initialized.", keypad);

    debug!("Initializing rotary encoder...");
    let mut rotenc_a = gpio.get_pin(5)?;
    let mut rotenc_b = gpio.get_pin(6)?;
    let rotenc_a = rotenc_a.as_input()?;
    let rotenc_b = rotenc_b.as_input()?;
    let mut rotenc = RotEnc::new(&*rotenc_a, &*rotenc_b);

    let mut rotenc_btn = gpio.get_pin(25)?;
    rotenc_btn.set_bias(GpioBias::PullUp)?;
    rotenc_btn.set_active_level(GpioActiveLevel::Low)?;
    let rotenc_btn = rotenc_btn.as_input()?;
    let mut rotenc_btn = TimedDebounce::new(&*rotenc_btn);

    debug!("{:?} initialized.", rotenc);

    debug!("Initializing PWM clock...");

    let mut pwm_clock = RawClockDriver::get_pwm()?;

    pwm_clock.set_enabled(false)?;
    sleep(Duration::from_millis(10));
    pwm_clock.set_mash_mode(MashMode::None)?;
    pwm_clock.set_source(pilock_gpio::clock::ClockSource::PllC)?; // 1 GHz
    pwm_clock.set_divisor(50.0)?; // 1/50 GHz = 20 MHz
    sleep(Duration::from_millis(10));
    pwm_clock.set_enabled(true)?;
    sleep(Duration::from_millis(10));

    debug!("{:?} initialized.", pwm_clock);

    debug!("Initializing PWM for audio...");

    // PWM 0/0 at GPIO 18

    let pwm = RawPwmDriver::new_mem(0)?;
    let mut pin_pwm = pwm.get_pin(0)?;
    pin_pwm.disable()?;

    // Manually set GPIO 18 to PWMW0 (ALT5)
    gpio.raw_set_pin_function(18, 0b010)?;

    let freq = note!("C4").as_freq_hz();
    let mut period_ns = (1_000_000_000.0 / freq) as u32;
    period_ns /= 4;
    pin_pwm.set_period_ns(period_ns)?;
    pin_pwm.set_duty_ns(period_ns / 2)?;
    pin_pwm.enable()?;
    sleep(Duration::from_secs(1));
    pin_pwm.disable()?;

    debug!("{:?} initialized.", pin_pwm);

    debug!("Trying to load config...");
    // let config = config::Config::try_load();
    let config = if let Some(config) = Config::try_load() {
        info!("Config loaded.");
        config
    } else {
        info!("Config not found. Using default");
        let mut config = Config::default();
        config.save()?;
        info!("Default config saved.");
        config
    };
    
    lcd.icon_booster_contrast(false, true, config.contrast.value())?;
    lcd.contrast_set(config.contrast.value())?;

    debug!("Password is {:?}.", config.password);

    info!("PiLock initialized.");

    sleep(Duration::from_secs(1));
    
    lcd.function_set_0(false, true, false, false)?;

    info!("Starting main loop...");
    
    let mut app = App::new(
        config,
        &mut lcd,
        // &mut keypad,
        &mut rotenc,
        &mut rotenc_btn,
        &mut *pin_pwm,
    );

    let mut last_update = Instant::now();
    loop {
        let now = Instant::now();
        app.update(last_update)?;
        last_update = now;
        
        // Sleep for 1/20th of a second
        thread::sleep(Duration::from_millis(1));
    }

    // Ok(())
}
