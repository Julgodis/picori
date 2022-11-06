use std::borrow::Borrow;
use std::marker::PhantomData;
use std::result::Result;

use crate::error::PicoriError;
use crate::error::StringEncodingError::*;
use crate::helper::read_extension::StringReadSupport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiDecoder<'x, I> {
    iter:    I,
    _marker: PhantomData<&'x ()>,
}

impl<I> AsciiDecoder<'_, I> {
    fn new<'x>(iter: I) -> AsciiDecoder<'x, I> {
        AsciiDecoder {
            iter,
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
    I: Iterator,
    I::Item: Borrow<u8>,
{
    type Item = Result<char, PicoriError>;

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
        I: Iterator + Clone,
        I::Item: Borrow<u8>,
    {
        AsciiDecoder::new(iter)
    }

    pub fn all(data: &[u8]) -> Result<String, PicoriError> { Self::iter(data.iter()).collect() }

    pub fn first(data: &[u8]) -> Result<String, PicoriError> {
        Self::iter(data.iter())
            .take_while(|c| match c {
                Ok(c) => *c != 0 as char,
                Err(_) => true,
            })
            .collect()
    }
}

pub trait AsciiIterator: Iterator + Clone + Sized {
    fn ascii<'b>(self) -> AsciiDecoder<'b, Self> { AsciiDecoder::new(self) }
}

impl<I> AsciiIterator for I
where
    I: Iterator + Clone + Sized,
    I::Item: Borrow<u8>,
{
}

impl StringReadSupport for Ascii {
    fn read_string(data: &[u8]) -> Result<String, PicoriError> { Self::first(data) }
}

// Tests
//

#[cfg(test)]
mod tests {
    use crate::helper::read_extension::StringReadSupport;
    use super::*;

    #[test]
    fn ok() {
        let result = (0_u8..=0x7f_u8).ascii().collect::<Result<String, _>>();
        assert!(result.is_ok());

        let ok = (0..=0x7f)
            .zip(result.unwrap().chars())
            .map(|(a, b)| a as u8 as char == b)
            .all(|x| x);
        assert!(ok);
    }

    #[test]
    fn err() {
        let result = (0x80..=0xff).ascii().all(|x| x.is_err());
        assert!(result);
    }

    #[test]
    fn first() {
        assert_eq!(&Ascii::first(b"abc\0def").unwrap()[..], "abc");
        assert!(&Ascii::first(b"abc\xffdef").is_err());
    }

    #[test]
    fn iter() {
        let data = b"abcdef";
        assert_eq!(
            Ascii::iter(data.iter())
                .map(|x| x.unwrap())
                .collect::<String>(),
            "abcdef"
        );
    }

    #[test]
    fn all() {
        let data = b"abc\0def";
        assert_eq!(&Ascii::all(data).unwrap()[..], "abc\0def");
    }

    #[test]
    fn read_string() {
        let data = b"abc\0def";
        assert_eq!(&Ascii::read_string(data).unwrap()[..], "abc");
    }
}
