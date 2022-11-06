use std::io::{Seek, SeekFrom};

use crate::Result;

pub trait Seeker {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
    fn position(&mut self) -> Result<u64>;
}

impl<Base> Seeker for Base
where
    Base: Seek + Sized,
{
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> { Ok(self.seek(pos)?) }

    #[inline]
    fn position(&mut self) -> Result<u64> { Ok(self.seek(SeekFrom::Current(0))? as u64) }
}
