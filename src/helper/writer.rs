use crate::{Error, Result};
use std::{io::Write, panic::Location};

use super::ParseStringEncoding;

pub trait Writer: Write {
    #[track_caller]
    #[inline]
    fn write_buffer(&mut self, buffer: &[u8]) -> Result<()> {
        self.write_buffer_tracked(buffer, Location::caller())
    }

    #[inline]
    fn write_buffer_tracked(
        &mut self,
        buffer: &[u8],
        caller: &'static std::panic::Location,
    ) -> Result<()> {
        match self.write_all(buffer) {
            Ok(..) => Ok(()),
            Err(io) => Err(Error::WriteFailed(buffer.len(), io, caller)),
        }
    }

    #[track_caller]
    #[inline]
    fn u8(&mut self, value: u8) -> Result<()> {
        self.write_buffer_tracked(&[value], Location::caller())
    }

    #[track_caller]
    #[inline]
    fn u8_array(&mut self, value: &[u8]) -> Result<()> {
        self.write_buffer_tracked(value, Location::caller())
    }

    #[track_caller]
    #[inline]
    fn lu16(&mut self, value: u16) -> Result<()> {
        self.write_buffer_tracked(&value.to_le_bytes(), Location::caller())
    }

    #[track_caller]
    #[inline]
    fn lu32(&mut self, value: u32) -> Result<()> {
        self.write_buffer_tracked(&value.to_le_bytes(), Location::caller())
    }

    #[track_caller]
    #[inline]
    fn bu16(&mut self, value: u16) -> Result<()> {
        self.write_buffer_tracked(&value.to_be_bytes(), Location::caller())
    }

    #[track_caller]
    #[inline]
    fn bu32(&mut self, value: u32) -> Result<()> {
        self.write_buffer_tracked(&value.to_be_bytes(), Location::caller())
    }

    #[inline]
    fn bu32_array(&mut self, value: &[u32]) -> Result<()> {
        for value in value {
            self.bu32(*value)?;
        }
        Ok(())
    }

    #[track_caller]
    fn str<const L: usize, E: ParseStringEncoding>(&mut self, data: &str) -> Result<()> {
        let mut buffer = [0u8; L];
        E::write_str(data, &mut buffer)?;
        self.write_buffer_tracked(&buffer, Location::caller())
    }
}

impl<Base: Write> Writer for Base {}

