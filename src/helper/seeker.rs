use std::io::{Seek, SeekFrom};

use crate::Result;

/// A helper trait for types that can seek.
pub trait Seeker {
    /// Seek to the given position.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;

    /// Get the current position.
    fn position(&mut self) -> Result<u64>;
}

/// Implementation of [`Seeker`] for all [`std::io::Seek`].
impl<Base> Seeker for Base
where
    Base: Seek + Sized,
{
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> { Ok(self.seek(pos)?) }

    #[inline]
    fn position(&mut self) -> Result<u64> { Ok(self.stream_position()?) }
}
