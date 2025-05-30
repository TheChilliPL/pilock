use pilock_gpio::clock::raw::RawClockDriver;
use pilock_gpio::clock::{ClockDriver, ClockSource, MashMode};
use pilock_gpio::lcd::ssd1803a::driver::{GpioSSD1803ADriver, SSD1803ADriver};
use pilock_gpio::pwm::{PwmDriver, PwmExtension, RawPwmDriver};
use pilock_gpio::raw::RawGpioDriver;
use pilock_gpio::{GpioActiveLevel, GpioBias, GpioDriveMode, GpioDriver};
use dotenv::dotenv;
use log::{debug, info};
use std::thread::sleep;
use std::time::Duration;
use sysinfo::System;
use pilock_gpio::lcd::hd44780::driver::CursorDirection;
use pilock_gpio::rotenc::{RotEnc, RotEncRotation};

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

    // let mut pwm_clock = RawClockDriver::get_pwm()?;
    //
    // pwm_clock.set_enabled(false)?;
    // sleep(Duration::from_millis(10));
    // assert!(!pwm_clock.get_busy()?);
    // pwm_clock.set_mash_mode(MashMode::None)?;
    // pwm_clock.set_source(ClockSource::PllD)?; // 1 GHz
    // pwm_clock.set_divisor(50.0)?;
    // assert_eq!(pwm_clock.divisor()?, 50.0); // 1/50 GHz = 20 MHz
    // // Delay for the clock to stabilize
    // sleep(Duration::from_millis(10));
    // pwm_clock.set_enabled(true)?;
    // sleep(Duration::from_millis(10));
    // assert!(pwm_clock.get_busy()?);
    //
    // // Manually set GPIO18 to PWM0
    // let _pwm_pin = gpio.get_pin(18)?;
    // gpio.raw_set_pin_function(18, 0b010)?; // GPIO18 ALT5: PWM0
    //
    // let pwm = RawPwmDriver::new_mem(0)?;
    // let mut pin_pwm = pwm.get_pin(0)?;
    // pin_pwm.disable()?;
    // pin_pwm.set_period(Duration::from_secs(1))?;
    // sleep(Duration::from_millis(10));
    // pin_pwm.set_duty(Duration::from_millis(500))?;
    // sleep(Duration::from_millis(10));
    // pin_pwm.enable()?;

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

    let contrast: u8 = 50; //31; // 0-63

    driver.contrast_set(contrast)?;
    driver.icon_booster_contrast(false, true, contrast)?;

    driver.cursor_shift(false, CursorDirection::Right)?;

    let mut rotenc_a = gpio.get_pin(5)?;
    let mut rotenc_b = gpio.get_pin(6)?;
    let rotenc_a = rotenc_a.as_input()?;
    let rotenc_b = rotenc_b.as_input()?;
    let mut rotenc = RotEnc::new(&*rotenc_a, &*rotenc_b);

    let mut rotenc_btn = gpio.get_pin(25)?;
    rotenc_btn.set_bias(GpioBias::PullUp)?;
    rotenc_btn.set_active_level(GpioActiveLevel::Low)?;
    let rotenc_btn = rotenc_btn.as_input()?;

    let mut num = 0i32;

    let mut changed = false;

    let mut frame = 0;

    loop {
        let rotation = rotenc.read()?;
        let (a, b) = rotenc.read_raw()?;

        if let Some(rot) = rotation {
            match rot {
                RotEncRotation::Clockwise => {
                    num += 1;
                }
                RotEncRotation::CounterClockwise => {
                    num -= 1;
                }
            }
            changed = true;
        }

        if rotenc_btn.read()? {
            num = 0;
            changed = true;
        }

        if changed && frame % 100 == 0 {
            driver.clear_display()?;

            let str = num.to_string();

            for char in str.chars() {
                driver.send_data(char as u8)?
            }

            changed = false;
        }

        sleep(Duration::from_millis(1));

        frame += 1;
    }

    Ok(())
}
