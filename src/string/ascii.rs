use std::borrow::Borrow;
use std::marker::PhantomData;

use crate::error::DecodingProblem::*;
use crate::helper::DeserializableStringEncoding;
use crate::Result;

pub struct AsciiDecoder<'x, I>
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
    iter:    <I as IntoIterator>::IntoIter,
    _marker: PhantomData<&'x ()>,
}

impl<I> AsciiDecoder<'_, I>
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
    fn new<'x>(iter: I) -> AsciiDecoder<'x, I> {
        AsciiDecoder {
            iter:    iter.into_iter(),
            _marker: PhantomData,
        }
    }

    fn decode_byte(byte: u8) -> Option<char> {
        match byte {
            // ASCII character
            0x00..=0x7f => Some(byte as char),
            // Invalid
            _ => None,
        }
    }
}

impl<I> Iterator for AsciiDecoder<'_, I>
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
    type Item = Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(byte) = self.iter.next() {
            let byte = byte.borrow();
            Some(match Self::decode_byte(*byte) {
                Some(c) => Ok(c),
                None => Err(InvalidByte(*byte).into()),
            })
        } else {
            None
        }
    }
}

pub struct Ascii {}

impl Ascii {
    pub fn iter<'iter, I>(iter: I) -> AsciiDecoder<'iter, I>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        AsciiDecoder::new(iter)
    }

    pub fn all<I>(iter: I) -> Result<String>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        Self::iter(iter).collect()
    }

    pub fn first<I>(iter: I) -> Result<String>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        Self::iter(iter)
            .take_while(|c| match c {
                Ok(c) => *c != 0 as char,
                Err(_) => true,
            })
            .collect()
    }
}

pub trait IteratorExt
where
    Self: IntoIterator + Sized,
    Self::Item: Borrow<u8> + Sized,
{
    fn ascii<'b>(self) -> AsciiDecoder<'b, Self> { AsciiDecoder::new(self) }
}

impl<I> IteratorExt for I
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
}

impl DeserializableStringEncoding for Ascii {
    fn deserialize_str<I>(iter: I) -> Result<String>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        Self::first(iter)
    }
}

// -------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_str() {
        let data = b"abc\0def";
        assert_eq!(Ascii::deserialize_str(data).unwrap(), "abc".to_string());
    }
}
