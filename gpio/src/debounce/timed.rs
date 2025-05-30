use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use crate::{GpioInput, GpioResult};

/// A debounced GPIO input that uses a timer to filter out noise.
pub struct TimedDebounce<'a> {
    input: &'a dyn GpioInput,
    state: AtomicBool,
    changed_since: Cell<Option<Instant>>,
    pub debounce_time: Duration,
}

impl <'a> TimedDebounce<'a> {
    pub fn new(input: &'a dyn GpioInput) -> Self {
        Self {
            input,
            state: AtomicBool::default(),
            changed_since: Cell::new(None),
            debounce_time: Duration::from_millis(50),
        }
    }
    
    pub fn with_debounce_time(mut self, debounce_time: Duration) -> Self {
        self.debounce_time = debounce_time;
        self
    }
}

impl Debug for TimedDebounce<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}(debounced)", self.input)
    }
}

impl GpioInput for TimedDebounce<'_> {
    fn read(&self) -> GpioResult<bool> {
        let previous_state = self.state.load(Ordering::Relaxed);
        let new_state = self.input.read()?;
        
        if let Some(instant) = self.changed_since.get() {
            if instant.elapsed() < self.debounce_time {
                if previous_state == new_state {
                    self.changed_since.set(None);
                }
                return Ok(previous_state);
            } else {
                self.changed_since.set(None);
                self.state.store(new_state, Ordering::Relaxed);
                return Ok(new_state);
            }
        }
        
        if previous_state != new_state {
            self.changed_since.set(Some(Instant::now()));
            self.state.store(new_state, Ordering::Relaxed);
        }
        
        Ok(previous_state)
    }
}
