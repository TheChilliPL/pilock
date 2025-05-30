use std::time::{Instant};
use time::Duration;
use pilock_gpio::GpioResult;
use pilock_gpio::keypad::{Keypad, KeypadKey};
use pilock_gpio::lcd::ssd1803a::driver::SSD1803ADriver;
use crate::config::Config;
use crate::utils::{CollectionExt, DisplayExt};

pub struct App<'a> {
    config: Config,
    state: AppState,
    lcd: &'a mut dyn SSD1803ADriver,
    keypad: &'a mut dyn Keypad<Key = KeypadKey>,
    easy_access: bool,
}

impl <'a> App<'a> {
    pub fn new(
        config: Config,
        lcd: &'a mut dyn SSD1803ADriver,
        keypad: &'a mut dyn Keypad<Key = KeypadKey>,
    ) -> App<'a> {
        App {
            config,
            state: AppState::default(),
            lcd,
            keypad,
            easy_access: false,
        }
    }

    pub fn update(&mut self, last_update: Instant) -> GpioResult<()> {
        let key = self.keypad.read()?.try_get_single().ok().copied();
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
                return Ok(());
            }
            AppState::Locked { .. } if self.easy_access => {
                self.state = AppState::LockedInEasyAccess;
                self.state.draw(self.lcd)?;
                return Ok(());
            }
            AppState::LockedInEasyAccess => {
                if key == Some(KeypadKey::KeyHash) {
                    self.state = AppState::Unlocked {
                        remaining: Duration::seconds(self.config.unlock_seconds.get() as i64),
                    };
                    self.state.draw(self.lcd)?;
                    return Ok(());
                }
                if key == Some(KeypadKey::KeyAsterisk) {
                    self.easy_access = false;
                    self.state = AppState::Locked {
                        input: Vec::new(),
                    };
                    self.state.draw(self.lcd)?;
                    return Ok(());
                }
            }
            AppState::Locked { ref mut input } => {
                if let Some(key) = key {
                    match key {
                        KeypadKey::KeyAsterisk => {
                            input.pop();
                            self.state.draw(self.lcd)?;
                        }
                        KeypadKey::KeyHash => {
                            if input.iter().collect::<String>() == self.config.password.iter().collect::<String>() {
                                self.state = AppState::Unlocked {
                                    remaining: Duration::seconds(self.config.unlock_seconds.get() as i64),
                                };
                            } else {
                                self.state = AppState::Locked {
                                    input: Vec::new(),
                                };
                            }
                            self.state.draw(self.lcd)?;
                        }
                        _ => {
                            if key.to_char().is_digit(10) {
                                if input.len() < 4 {
                                    input.push(key.to_char());
                                    self.state.draw(self.lcd)?;
                                }
                            }
                        }
                    }
                }
            }
            AppState::Unlocked { ref mut remaining } => {
                *remaining -= duration;
                
                if remaining.is_negative() {
                    self.state = if self.easy_access {
                        AppState::LockedInEasyAccess
                    } else {
                        AppState::Locked {
                            input: Vec::new(),
                        }
                    };
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
                lcd.set_cursor(3, 0)?;
                lcd.print("* del / # unlock")?;
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
