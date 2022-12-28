use std::io::{Seek, SeekFrom};
use std::panic::Location;

use crate::{Error, Result};

/// A helper trait for types that can seek.
pub trait Seeker: Seek {
    /// Seek to the given position.
    #[track_caller]
    fn goto(&mut self, pos: u64) -> Result<u64> {
        self.goto_tracked(pos, Location::caller())
    }

    /// Seek to the given position.
    #[inline]
    fn goto_tracked(&mut self, pos: u64, caller: &'static Location) -> Result<u64> {
        match self.seek(SeekFrom::Start(pos)) {
            Ok(_) => Ok(pos),
            Err(e) => Err(Error::SeekFailed(e, caller)),
        }
    }

    /// Get the current position.
    #[track_caller]
    fn position(&mut self) -> Result<u64> {
        self.position_tracked(Location::caller())
    }

    /// Get the current position.
    #[inline]
    fn position_tracked(&mut self, caller: &'static Location) -> Result<u64> {
        match self.stream_position() {
            Ok(pos) => Ok(pos),
            Err(e) => Err(Error::SeekFailed(e, caller)),
        }
    }
}

impl Seeker for std::fs::File {}
impl<T: Seeker> Seeker for std::io::BufReader<T> {}
impl<T> Seeker for std::io::Cursor<T>
where
    Self: Seek,
    T: AsRef<[u8]>,
{
}

/*
/// Implementation of [`Seeker`] for all [`std::io::Seek`].
impl<Base> Seeker for Base
where
    Base: Seek + Sized,
{
    #[inline]
    fn goto_tracked(&mut self, pos: u64, caller: &'static Location) -> Result<u64> {
        match self.seek(SeekFrom::Start(pos)) {
            Ok(_) => Ok(pos),
            Err(e) => Err(Error::SeekFailed(e, caller)),
        }
    }

    #[inline]
    fn position_tracked(&mut self, caller: &'static Location) -> Result<u64> {
        match self.stream_position() {
            Ok(pos) => Ok(pos),
            Err(e) => Err(Error::SeekFailed(e, caller)),
        }
    }
}
*/
