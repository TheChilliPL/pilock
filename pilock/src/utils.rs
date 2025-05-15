use thiserror::Error;

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
