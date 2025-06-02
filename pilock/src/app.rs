use std::time::{Instant};
use time::Duration;
use pilock_gpio::{GpioInput, GpioResult};
use pilock_gpio::keypad::{Keypad, KeypadKey};
use pilock_gpio::lcd::ssd1803a::driver::SSD1803ADriver;
use pilock_gpio::rotenc::RotEnc;
use crate::config::Config;
use crate::utils::{CollectionExt, DisplayExt};

pub struct App<'a> {
    config: Config,
    state: AppState,
    lcd: &'a mut dyn SSD1803ADriver,
    // keypad: &'a mut dyn Keypad<Key = KeypadKey>,
    encoder: &'a mut RotEnc<'a>,
    ok_btn: &'a mut dyn GpioInput,
    easy_access: bool,

    prev_ok_state: bool,
}

impl <'a> App<'a> {
    pub fn new(
        config: Config,
        lcd: &'a mut dyn SSD1803ADriver,
        // keypad: &'a mut dyn Keypad<Key = KeypadKey>,
        encoder: &'a mut RotEnc<'a>,
        ok_btn: &'a mut dyn GpioInput,
    ) -> App<'a> {
        App {
            config,
            state: AppState::default(),
            lcd,
            // keypad,
            encoder,
            ok_btn,
            easy_access: false,
            prev_ok_state: false,
        }
    }

    pub fn update(&mut self, last_update: Instant) -> GpioResult<()> {
        // let key = self.keypad.read()?.try_get_single().ok().copied();
        let rotation = self.encoder.read()?;
        let ok_pressed = self.ok_btn.read()?;
        const STATE_RELEASED: i32 = 2;
        const STATE_PRESSED: i32 = 1;

        let ok_state = match (self.prev_ok_state, ok_pressed) {
            (true, false) => STATE_RELEASED, // Button released
            (false, true) => STATE_PRESSED, // Button pressed
            _ => 0, // No change
        };
        self.prev_ok_state = ok_pressed;
        let duration = last_update.elapsed();

        match self.state {
            AppState::Starting => {
                self.state = if self.easy_access {
                    AppState::LockedInEasyAccess
                } else {
                    AppState::Locked {
                        input: Vec::new(),
                    }
                };
                self.state.draw(self.lcd)?;
            }
            AppState::LockedInEasyAccess if !self.easy_access => {
                self.state = AppState::Locked {
                    input: Vec::new(),
                };
                self.state.draw(self.lcd)?;
            }
            AppState::Locked { .. } if self.easy_access => {
                self.state = AppState::LockedInEasyAccess;
                self.state.draw(self.lcd)?;
            }
            AppState::LockedInEasyAccess => {
                // if key == Some(KeypadKey::KeyHash) {
                if ok_state == 2 {
                    self.state = AppState::Unlocked {
                        remaining: Duration::seconds(self.config.unlock_seconds.get() as i64),
                    };
                    self.state.draw(self.lcd)?;
                    return Ok(());
                }
                // if key == Some(KeypadKey::KeyAsterisk) {
                //     self.easy_access = false;
                //     self.state = AppState::Locked {
                //         input: Vec::new(),
                //     };
                //     self.state.draw(self.lcd)?;
                //     return Ok(());
                // }
            }
            AppState::Locked { ref mut input } => {
                // if let Some(key) = key {
                //     match key {
                //         KeypadKey::KeyAsterisk => {
                //             input.pop();
                //             self.state.draw(self.lcd)?;
                //         }
                //         KeypadKey::KeyHash => {
                //             if input.iter().collect::<String>() == self.config.password.iter().collect::<String>() {
                //                 self.state = AppState::Unlocked {
                //                     remaining: Duration::seconds(self.config.unlock_seconds.get() as i64),
                //                 };
                //             } else {
                //                 self.state = AppState::Locked {
                //                     input: Vec::new(),
                //                 };
                //             }
                //             self.state.draw(self.lcd)?;
                //         }
                //         _ => {
                //             if key.to_char().is_digit(10) {
                //                 if input.len() < 4 {
                //                     input.push(key.to_char());
                //                     self.state.draw(self.lcd)?;
                //                 }
                //             }
                //         }
                //     }
                // }
                match rotation {
                    Some(rot) => {
                        let rot = match rot {
                            pilock_gpio::rotenc::RotEncRotation::Clockwise => 1,
                            pilock_gpio::rotenc::RotEncRotation::CounterClockwise => -1,
                        };
                        if input.len() < 1 {
                            input.push('0');
                        } else {
                            let last_digit = input.last_mut().unwrap();
                            let last_digit_value = last_digit.to_digit(10).unwrap_or(0) as i8;
                            let new_digit_value = (last_digit_value + rot).rem_euclid(10) as u8;
                            *last_digit = std::char::from_digit(new_digit_value as u32, 10).unwrap();
                        }
                        self.state.draw(self.lcd)?;
                    }
                    None => {
                        if ok_state == STATE_PRESSED {
                            if input.len() >= self.config.password.len() {
                                if input.iter().collect::<String>() == self.config.password.iter().collect::<String>() {
                                    self.state = AppState::Unlocked {
                                        remaining: Duration::seconds(self.config.unlock_seconds.get() as i64),
                                    };
                                } else {
                                    self.state = AppState::Locked {
                                        input: Vec::new(),
                                    };
                                }
                            } else {
                                input.push('0');
                            }
                            self.state.draw(self.lcd)?;
                        }
                    }
                }
            }
            AppState::Unlocked { ref mut remaining } => {
                let prev_sec = remaining.whole_seconds();
                *remaining -= duration;
                let now_sec = remaining.whole_seconds();
                
                if remaining.is_negative() {
                    self.state = if self.easy_access {
                        AppState::LockedInEasyAccess
                    } else {
                        AppState::Locked {
                            input: Vec::new(),
                        }
                    };
                    self.state.draw(self.lcd)?;
                } else if prev_sec != now_sec {
                    self.state.draw(self.lcd)?;
                }
            }
        }

        // self.state.draw(self.lcd)?;
        Ok(())
    }
}

#[derive(Default)]
pub enum AppState {
    #[default]
    Starting,
    LockedInEasyAccess,
    Locked {
        input: Vec<char>,
    },
    Unlocked {
        remaining: Duration,
    },
}

impl AppState {
    fn draw(&self, lcd: &mut dyn SSD1803ADriver) -> GpioResult<()> {
        lcd.clear_display()?;
        match self {
            AppState::Starting => {}
            AppState::LockedInEasyAccess => {
                lcd.set_cursor(0, 0)?;
                lcd.print("Easy access")?;
                lcd.set_cursor(1, 0)?;
                lcd.print("Press # to unlock")?;
            }
            AppState::Locked { input } => {
                lcd.set_cursor(0, 0)?;
                lcd.print("Enter password:")?;
                lcd.set_cursor(1, 0)?;
                lcd.print(&input.iter().collect::<String>())?;
            }
            AppState::Unlocked { remaining } => {
                lcd.set_cursor(0, 0)?;
                lcd.print("Unlocked!")?;
                lcd.set_cursor(1, 0)?;
                lcd.print(&format!("{}s remaining", remaining.whole_seconds()))?;
            }
        }
        Ok(())
    }
}
