use std::num::NonZero;
use crate::{GpioInput, GpioResult};

/// Represents the direction of rotation for a rotary encoder.
pub enum RotEncRotation {
    Clockwise,
    CounterClockwise,
}

/// A rotary encoder driver that reads the state of two GPIO pins to determine the direction of rotation.
/// 
/// Does not provide a button, so you may want to use a separate GPIO input for that.
#[derive(Debug)]
pub struct RotEnc<'a> {
    pub pin_a: &'a dyn GpioInput,
    pub pin_b: &'a dyn GpioInput,
    state: (bool, bool),
    ticks_per_rotation: u8,
    tick_count: i8,
    reading_limit: u32,
    reading_start: Option<NonZero<u32>>,
    ticks: NonZero<u32>,
}

impl<'a> RotEnc<'a> {
    const STATES_CLOCKWISE: [(bool, bool); 4] = [
        (false, false),
        (true, false),
        (true, true),
        (false, true),
    ];
    
    pub fn new(pin_a: &'a dyn GpioInput, pin_b: &'a dyn GpioInput) -> Self {
        let mut rot_enc = RotEnc {
            pin_a, pin_b,
            state: (false, false),
            ticks_per_rotation: 2,
            tick_count: 0,
            reading_limit: 200,
            reading_start: None,
            ticks: unsafe { NonZero::new_unchecked(1) },
        };
        rot_enc.state = rot_enc.read_raw().unwrap_or((false, false));
        rot_enc
    }

    pub fn read_raw(&self) -> GpioResult<(bool, bool)> {
        let a = self.pin_a.read()?;
        let b = self.pin_b.read()?;
        Ok((a, b))
    }
    
    pub fn read(&mut self) -> GpioResult<Option<RotEncRotation>> {
        let mut tick = self.ticks.get().wrapping_add(1);
        if tick == 0 {
            tick = 1;
            self.reading_start = None;
        }
        self.ticks = unsafe { NonZero::new_unchecked(tick) };
        
        let previous_state = self.state;
        let current_state = self.read_raw()?;
        
        self.state = current_state;
        
        if current_state == previous_state {
            return Ok(None);
        }
        
        let previous_index = Self::STATES_CLOCKWISE.iter().position(|&s| s == previous_state);
        let current_index = Self::STATES_CLOCKWISE.iter().position(|&s| s == current_state);
        
        if let (Some(prev_idx), Some(curr_idx)) = (previous_index, current_index) {
            if (curr_idx + 1) % Self::STATES_CLOCKWISE.len() == prev_idx {
                self.tick_count -= 1;
                self.reading_start.get_or_insert(self.ticks);
            } else if (prev_idx + 1) % Self::STATES_CLOCKWISE.len() == curr_idx {
                self.tick_count += 1;
                self.reading_start.get_or_insert(self.ticks);
            }
        }
        
        if let Some(start) = self.reading_start {
            if start.get() + self.reading_limit <= tick {
                self.reading_start = None;
                
                self.tick_count = 0;
            }
        }
        
        if self.tick_count >= self.ticks_per_rotation as i8 {
            self.tick_count = 0;
            self.reading_start = None;
            Ok(Some(RotEncRotation::Clockwise))
        } else if self.tick_count <= -(self.ticks_per_rotation as i8) {
            self.tick_count = 0;
            self.reading_start = None;
            Ok(Some(RotEncRotation::CounterClockwise))
        } else {
            Ok(None)
        }
    }
}

