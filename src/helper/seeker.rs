use std::io::{Seek, SeekFrom};

use crate::PicoriError;

pub trait Seeker {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, PicoriError>;
    fn position(&mut self) -> Result<u64, PicoriError>;
}

impl<Base> Seeker for Base
where
    Base: Seek + Sized,
{
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, PicoriError> { Ok(self.seek(pos)?) }

    #[inline]
    fn position(&mut self) -> Result<u64, PicoriError> {
        Ok(self.seek(SeekFrom::Current(0))? as u64)
    }
}
