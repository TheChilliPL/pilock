mod gpio;

use std::hint::spin_loop;
use std::ops::Add;
use std::pin::pin;
use std::ptr::NonNull;
use crate::gpio::{GpioActiveLevel, GpioBias, GpioBusInput, GpioDriveMode, GpioDriver, GpioInput};
use crate::gpio::gpiod::GpiodDriver;
use crate::gpio::lcd::hd44780::driver::{GpioHD44780Driver, HD44780Driver};
use dotenv::dotenv;
use gpiod::Chip;
use log::{debug, info};
use std::thread::sleep;
use std::time::Duration;
use sysinfo::System;
use time::OffsetDateTime;
use crate::gpio::clock::{ClockDriver, ClockSource, MashMode};
use crate::gpio::clock::raw::RawClockDriver;
use crate::gpio::debounce::TimedDebounce;
use crate::gpio::lcd::ssd1803a::driver::{GpioSSD1803ADriver, SSD1803ADriver};
use crate::gpio::lcd::ssd1803a::driver::DoubleHeightMode::DoubleBottom;
use crate::gpio::pwm::{PwmDriver, PwmExtension, RawPwmDriver, SysfsPwmDriver};
use crate::gpio::raw::RawGpioDriver;
use crate::gpio::soft::{SoftGpioBus, SoftGpioBusInput};

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

    // let gpio = RawGpioDriver::new_gpiomem()?;
    let gpio = RawGpioDriver::new_mem()?;
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

    let mut pwm_clock = RawClockDriver::get_pwm()?;

    pwm_clock.set_enabled(false)?;
    sleep(Duration::from_millis(10));
    assert!(!pwm_clock.get_busy()?);
    pwm_clock.set_mash_mode(MashMode::None)?;
    pwm_clock.set_source(ClockSource::PllD)?; // 1 GHz
    pwm_clock.set_divisor(50.0)?;
    assert_eq!(pwm_clock.divisor()?, 50.0); // 1/50 GHz = 20 MHz
    // Delay for the clock to stabilize
    sleep(Duration::from_millis(10));
    pwm_clock.set_enabled(true)?;
    sleep(Duration::from_millis(10));
    assert!(pwm_clock.get_busy()?);

    // Manually set GPIO18 to PWM0
    let _pwm_pin = gpio.get_pin(18)?;
    gpio.raw_set_pin_function(18, 0b010)?; // GPIO18 ALT5: PWM0

    let pwm = RawPwmDriver::new_mem(0)?;
    let mut pin_pwm = pwm.get_pin(0)?;
    pin_pwm.disable()?;
    pin_pwm.set_period(Duration::from_secs(1))?;
    sleep(Duration::from_millis(10));
    pin_pwm.set_period(Duration::from_millis(500))?;
    sleep(Duration::from_millis(10));
    pin_pwm.enable()?;

    // loop {
    //     spin_loop();
    // }

    // {
    let mut data_bus = gpio.get_pin_bus(bus_pins)?;

    let mut driver = GpioSSD1803ADriver::new_4bit(
        None,
        &*pin_e_out,
        Some(&*pin_rw_out),
        &*pin_rs_out,
        &mut *data_bus,
    );

    driver.init(4)?;

    let contrast: u8 = 31; // 0-63

    driver.contrast_set(contrast)?;
    driver.icon_booster_contrast(false, true, contrast)?;
    //
    //     let str = "Hi PiLock 4-bit";
    //
    //     for c in str.chars() {
    //         driver.send_data(c as u8)?;
    //     }
    //
    //     driver.set_ddram_address(0x20)?;
    //
    //     let str = System::cpu_arch();
    //
    //     for c in str.chars() {
    //         driver.send_data(c as u8)?
    //     }
    //
    //     driver.double_height_bias_dot_shift(DoubleBottom, Default::default(), false)?;
    //     driver.function_set_0(false, true, true, false)?;
    //
    //     loop {
    //         let time = OffsetDateTime::now_local()?;
    //         let (h, m, s) = time.to_hms();
    //
    //         driver.set_ddram_address(0x36)?;
    //
    //         let str = format!("{:02}:{:02}:{:02}", h, m, s);
    //         for c in str.chars() {
    //             driver.send_data(c as u8)?;
    //         }
    //
    //         // Delay until the start of the next second
    //         let next_second = time.clone()
    //             .add(Duration::from_secs(1))
    //             .replace_nanosecond(0)?;
    //         sleep(Duration::try_from(next_second - time)?);
    //     }
    // }

    // Keypad test

    let mut cols = [ 23, 24, 25, 12 ];
    let mut rows = [ 5, 6, 8, 7 ];
    cols.reverse();
    rows.reverse();
    let mut cols = gpio.get_pin_bus(cols)?;
    let mut rows = gpio.get_pin_bus(rows)?;

    cols.set_active_level(GpioActiveLevel::Low)?;
    cols.set_drive_mode(GpioDriveMode::OpenDrain)?;
    rows.set_active_level(GpioActiveLevel::Low)?;
    rows.set_bias(GpioBias::PullUp)?;

    let cols = cols.as_output()?;
    let rows = rows.as_input()?;

    let mut keypad = [[false; 4]; 4];

    let keypad_chars = [
        ['1', '2', '3', 'A'],
        ['4', '5', '6', 'B'],
        ['7', '8', '9', 'C'],
        ['*', '0', '#', 'D'],
    ];

    let mut input: Vec<char> = Vec::with_capacity(8);

    let mut last_pressed = None;

    loop {
        let mut pressed = None;

        for col in 0..4 {
            let nibble = 1 << (3 - col);
            cols.write_nibble(nibble)?;
            sleep(Duration::from_millis(10));
            let value = rows.read_nibble()?;
            for row in 0..4 {
                let value = value >> (3 - row) & 1;
                keypad[row][col] = value == 1;

                if value == 1 {
                    if pressed.is_none() {
                        pressed = Some(keypad_chars[row][col]);
                    } else {
                        continue;
                    }
                }
            }
            // debug!("Keypad pin {}: {:04b} (written {:04b})", pin, value, nibble);
        }

        if let Some(pressed) = pressed {
            if last_pressed != Some(pressed) {
                last_pressed = Some(pressed);
                debug!("Keypad pressed: {}", pressed);
                match pressed {
                    '*' => {
                        input.pop();
                    }
                    '#' => {
                        if input.len() > 0 {
                            let str = input.iter().collect::<String>();
                            debug!("Keypad input: {}", str);
                            input.clear();
                        }
                    }
                    _ => {
                        if input.capacity() - input.len() > 0 {
                            input.push(pressed);
                        }
                    }
                }
            }
        } else {
            last_pressed = None;
        }

        // for row in 0..4 {
        //     driver.set_ddram_address(0x20 * row)?;
        //     let row = row as usize;
        //     for col in 0..4 {
        //         if keypad[row][col] {
        //             driver.send_data(keypad_chars[row][col] as u8)?;
        //         } else {
        //             driver.send_data('-' as u8)?;
        //         }
        //         driver.send_data(' ' as u8)?;
        //     }
        // }

        driver.clear_display()?;

        for i in 0..input.capacity() {
            let char = input.get(i).cloned().unwrap_or('_');
            driver.send_data(char as u8)?;
            driver.send_data(' ' as u8)?;
        }

        sleep(Duration::from_millis(1000 / 30));
    }

    Ok(())
}
