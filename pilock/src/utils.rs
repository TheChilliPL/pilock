use std::ops::RangeBounds;
use log::warn;
use thiserror::Error;
use pilock_gpio::{GpioError, GpioResult};
use pilock_gpio::lcd::ssd1803a::driver::SSD1803ADriver;

#[derive(Copy, Clone, Debug, Error)]
pub enum TryGetSingleError {
    #[error("collection is empty")]
    Empty,
    #[error("collection has more than one item")]
    MoreThanOne,
}

pub trait CollectionExt {
    type Item;

    fn try_get_single(&self) -> Result<&Self::Item, TryGetSingleError>;
    fn try_get_single_mut(&mut self) -> Result<&mut Self::Item, TryGetSingleError>;
}

impl <T> CollectionExt for Vec<T> {
    type Item = T;

    fn try_get_single(&self) -> Result<&T, TryGetSingleError> {
        match self.len() {
            0 => Err(TryGetSingleError::Empty),
            1 => Ok(&self[0]),
            _ => Err(TryGetSingleError::MoreThanOne),
        }
    }

    fn try_get_single_mut(&mut self) -> Result<&mut T, TryGetSingleError> {
        match self.len() {
            0 => Err(TryGetSingleError::Empty),
            1 => Ok(&mut self[0]),
            _ => Err(TryGetSingleError::MoreThanOne),
        }
    }
}

pub trait WithinExt {
    fn within(&self, range: impl RangeBounds<Self>) -> bool;
}

impl <T: PartialOrd<T>> WithinExt for T {
    fn within(&self, range: impl RangeBounds<Self>) -> bool {
        range.contains(&self)
    }
}

pub trait DisplayExt {
    fn print(&mut self, s: &str) -> GpioResult<()>;
    fn set_cursor(&mut self, row: usize, col: usize) -> GpioResult<()>;
}

impl <T: ?Sized + SSD1803ADriver> DisplayExt for T {
    fn print(&mut self, s: &str) -> GpioResult<()> {
        for c in s.chars() {
            if c.is_ascii() {
                self.send_data(c as u8)?;
            } else {
                warn!("Non-ASCII character: {}", c);
                self.send_data(b'?')?
            }
        }
        Ok(())
    }

    fn set_cursor(&mut self, row: usize, col: usize) -> GpioResult<()> {
        if !row.within(0..4) || !col.within(0..20) {
            return Err(GpioError::InvalidArgument);
        }
        self.set_ddram_address((col + 0x20 * row) as u8)
    }
}
