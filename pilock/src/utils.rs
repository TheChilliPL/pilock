//! Extension traits for various types, providing additional functionality.

use std::ops::RangeBounds;
use log::warn;
use thiserror::Error;
use pilock_gpio::{GpioError, GpioResult};
use pilock_gpio::lcd::ssd1803a::driver::SSD1803ADriver;

// A custom error type for when trying to get a single item from a collection that is expected to have exactly one item.
#[derive(Copy, Clone, Debug, Error)]
pub enum TryGetSingleError {
    /// The collection is empty.
    #[error("collection is empty")]
    Empty,
    /// The collection has more than one item.
    #[error("collection has more than one item")]
    MoreThanOne,
}

/// An extension trait for collections.
pub trait CollectionExt {
    type Item;

    /// Attempts to get a single item from the collection as a reference.
    /// 
    /// Returns an error if the collection is empty or has more than one item.
    fn try_get_single(&self) -> Result<&Self::Item, TryGetSingleError>;
    
    /// Attempts to get a single item from the collection as a mutable reference.
    /// 
    /// Returns an error if the collection is empty or has more than one item.
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

/// An extension trait for types that can be checked if they are within a range.
pub trait WithinExt {
    /// Checks if the value is within the specified range.
    fn within(&self, range: impl RangeBounds<Self>) -> bool;
}

impl <T: PartialOrd<T>> WithinExt for T {
    fn within(&self, range: impl RangeBounds<Self>) -> bool {
        range.contains(&self)
    }
}

/// An extension trait for the SSD1803A display driver, providing additional functionality.
/// 
/// Not suitable to put in the [pilock_gpio] crate, as it is a specific implementation that
/// might not always work. To circumvent that issue, a higher-level API that manages state could be defined.
pub trait DisplayExt {
    /// Prints a string to the display.
    /// 
    /// Does **not** support non-ASCII characters or new lines.
    fn print(&mut self, s: &str) -> GpioResult<()>;
    
    /// Sets the cursor position on the display.
    /// 
    /// The position is specified in terms of rows and columns, where the first row is 0 and the first column is 0.
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
