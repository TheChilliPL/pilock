use crate::notes::MusicalNote;
use pilock_music_proc_macro::note;
use std::time::{Instant};
use log::{info, warn};
use time::Duration;
use pilock_gpio::{GpioInput, GpioResult};
use pilock_gpio::keypad::{Keypad, KeypadKey};
use pilock_gpio::lcd::ssd1803a::driver::SSD1803ADriver;
use pilock_gpio::pwm::{PwmExtension, PwmPin};
use pilock_gpio::rotenc::RotEnc;
use crate::config::Config;
use crate::melody;
use crate::notes::{Melody, MelodyExt};
use crate::utils::{CollectionExt, DisplayExt};

pub struct App<'a> {
    config: Config,
    state: AppState,
    lcd: &'a mut dyn SSD1803ADriver,
    // keypad: &'a mut dyn Keypad<Key = KeypadKey>,
    encoder: &'a mut RotEnc<'a>,
    audio_pwm: &'a mut dyn PwmPin,
    ok_btn: &'a mut dyn GpioInput,
    easy_access: bool,

    prev_ok_state: bool,

    audio_state: Option<(Melody, Duration)>,
}

impl <'a> App<'a> {
    pub fn new(
        config: Config,
        lcd: &'a mut dyn SSD1803ADriver,
        // keypad: &'a mut dyn Keypad<Key = KeypadKey>,
        encoder: &'a mut RotEnc<'a>,
        ok_btn: &'a mut dyn GpioInput,
        audio_pwm: &'a mut dyn PwmPin,
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
            audio_pwm,
            audio_state: None,
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
        
        if let Err(e) = self.update_audio(duration.try_into().expect("Invalid duration")) {
            warn!("Failed to update audio: {}", e);
        }

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
                                let input_pin = input.iter().collect::<String>();
                                if input_pin == self.config.password.iter().collect::<String>() {
                                    self.state = AppState::Unlocked {
                                        remaining: Duration::seconds(self.config.unlock_seconds.get() as i64),
                                    };
                                    self.start_unlock_melody();
                                } else {
                                    self.state = AppState::Locked {
                                        input: Vec::new(),
                                    };
                                    if input_pin == "0915" {
                                        self.start_megalovania();
                                    } else {
                                        self.start_fail_melody();
                                    }
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

    pub fn start_melody(&mut self, melody: Melody) {
        if let Some(_) = &self.audio_state {
            warn!("Audio is already playing, overwriting audio state.");
        }
        self.audio_state = Some((melody, Duration::ZERO));
        info!("Starting melody.");
    }
    
    pub fn start_unlock_melody(&mut self) {
        let melody = melody![
            "G4" for 150 ms,
            pause for 50 ms,
            "G5" for 150 ms,
            pause for 50 ms,
            "E5" for 150 ms,
            pause for 50 ms,
            "C5" for 150 ms,
            pause for 50 ms,
            "D5" for 150 ms,
            pause for 50 ms,
            "G5" for 300 ms
        ];
        
        self.start_melody(melody);
    }
    
    pub fn start_fail_melody(&mut self) {
        let melody = melody![
            "G4" for 500 ms,
            pause for 50 ms,
            "G4" for 500 ms,
            pause for 50 ms,
            "G4" for 500 ms,
            pause for 50 ms,
            "D#4" for 350 ms,
            pause for 50 ms,
            "A#4" for 150 ms,
            pause for 50 ms,
            "G4" for 500 ms,
            pause for 50 ms,
            "D#4" for 350 ms,
            pause for 50 ms,
            "A#4" for 150 ms,
            pause for 50 ms,
            "G4" for 1000 ms
        ];
        self.start_melody(melody);
    }

    pub fn start_megalovania(&mut self) {
        let melody = melody![
            "D4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "C4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "B3" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "A#3" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "D4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "C4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "B3" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "A#3" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "D4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "C4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "B3" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "A#3" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "D4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "C4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "B3" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "A#3" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "F4" for 200 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            pause for 200 ms,
            "D4" for 100 ms,
            pause for 100 ms,
            "F4" for 400 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            pause for 200 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "F4" for 200 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "C5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "D5" for 100 ms,
            "A4" for 100 ms,
            "D5" for 100 ms,
            "C5" for 100 ms,
            pause for 800 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "A4" for 200 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 400 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "A4" for 200 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "C5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            pause for 100 ms,
            "E4" for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "C5" for 100 ms,
            pause for 1600 ms,
            "F4" for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            "D4" for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            pause for 800 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            "C5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            "D4" for 100 ms,
            "E4" for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "C5" for 100 ms,
            pause for 200 ms,
            "C#5" for 100 ms,
            pause for 100 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            pause for 800 ms,
            "F3" for 100 ms,
            pause for 100 ms,
            "G3" for 100 ms,
            pause for 100 ms,
            "A3" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "E4" for 100 ms,
            pause for 300 ms,
            "D4" for 100 ms,
            pause for 300 ms,
            "E4" for 100 ms,
            pause for 300 ms,
            "F4" for 100 ms,
            pause for 300 ms,
            "G4" for 100 ms,
            pause for 300 ms,
            "E4" for 100 ms,
            pause for 300 ms,
            "A4" for 100 ms,
            pause for 700 ms,
            "A4" for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F#4" for 100 ms,
            "F4" for 100 ms,
            "E4" for 100 ms,
            "D#4" for 100 ms,
            "D4" for 100 ms,
            "C#4" for 100 ms,
            pause for 700 ms,
            "D#4" for 100 ms,
            pause for 1500 ms,
            "F4" for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            "D4" for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            "D4" for 100 ms,
            "E4" for 100 ms,
            "G4" for 100 ms,
            pause for 800 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "C5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            "D4" for 100 ms,
            "E4" for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 100 ms,
            "C5" for 100 ms,
            pause for 100 ms,
            "C#5" for 100 ms,
            pause for 100 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            pause for 600 ms,
            "F3" for 100 ms,
            pause for 100 ms,
            "G3" for 100 ms,
            pause for 100 ms,
            "A3" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "E4" for 100 ms,
            pause for 300 ms,
            "D4" for 100 ms,
            pause for 300 ms,
            "E4" for 100 ms,
            pause for 300 ms,
            "F4" for 100 ms,
            pause for 300 ms,
            "G4" for 100 ms,
            pause for 300 ms,
            "E4" for 100 ms,
            pause for 300 ms,
            "A4" for 100 ms,
            pause for 700 ms,
            "A4" for 100 ms,
            "G#4" for 100 ms,
            "G4" for 100 ms,
            "F#4" for 100 ms,
            "F4" for 100 ms,
            "E4" for 100 ms,
            "D#4" for 100 ms,
            "D4" for 100 ms,
            "C#4" for 100 ms,
            pause for 700 ms,
            "D#4" for 100 ms,
            pause for 900 ms,
            "B3" for 100 ms,
            pause for 1100 ms,
            "F4" for 100 ms,
            pause for 300 ms,
            "E4" for 100 ms,
            pause for 700 ms,
            "D4" for 100 ms,
            pause for 700 ms,
            "F4" for 100 ms,
            pause for 3100 ms,
            "B3" for 100 ms,
            pause for 1100 ms,
            "F4" for 100 ms,
            pause for 300 ms,
            "E4" for 100 ms,
            pause for 700 ms,
            "D4" for 100 ms,
            pause for 1100 ms,
            "D4" for 100 ms,
            pause for 3100 ms,
            "B3" for 100 ms,
            pause for 1100 ms,
            "F4" for 100 ms,
            pause for 300 ms,
            "E4" for 100 ms,
            pause for 700 ms,
            "D4" for 100 ms,
            pause for 700 ms,
            "F4" for 100 ms,
            pause for 3100 ms,
            "B3" for 100 ms,
            pause for 1100 ms,
            "F4" for 100 ms,
            pause for 300 ms,
            "E4" for 100 ms,
            pause for 700 ms,
            "D4" for 100 ms,
            pause for 700 ms,
            "D4" for 100 ms,
            pause for 1300 ms,
            "D4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "D4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "C#4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "C4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "D4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "D4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "C#4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
            "C4" for 200 ms,
            "D5" for 100 ms,
            pause for 100 ms,
            "A4" for 100 ms,
            pause for 200 ms,
            "G#4" for 100 ms,
            pause for 100 ms,
            "G4" for 100 ms,
            pause for 100 ms,
            "F4" for 100 ms,
            pause for 100 ms,
            "D4" for 100 ms,
            "F4" for 100 ms,
            "G4" for 100 ms,
        ];
        self.start_melody(melody);
    }

    fn update_audio(&mut self, time_elapsed: Duration) -> GpioResult<()> {
        if let Some((melody, elapsed)) = &mut self.audio_state {
            if *elapsed >= melody.duration() {
                self.audio_pwm.disable()?;
                self.audio_state = None;
                info!("Audio playback finished.");
                return Ok(());
            }

            match melody.get_note_at(*elapsed) {
                Some(note) => {
                    let mut period = core::time::Duration::from_secs_f64(note.as_freq_hz().recip());
                    period /= 4; // Frequency *4 to increase octave
                    let duty = period / 2; // 50% duty cycle
                    if !self.audio_pwm.is_enabled()? || self.audio_pwm.period() != Ok(period) {
                        // self.audio_pwm.disable()?;
                        self.audio_pwm.set_period(period)?;
                        self.audio_pwm.set_duty(duty)?;
                        self.audio_pwm.enable()?;
                    }
                }
                None => {
                    if self.audio_pwm.is_enabled()? {
                        self.audio_pwm.disable()?;
                    }
                }
            }

            *elapsed += time_elapsed; // Update by 10 ms
            self.audio_state = Some((melody.clone(), *elapsed));
        }

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
