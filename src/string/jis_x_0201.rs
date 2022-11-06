//! `JIS X 0201` (ANK) is a single-byte encoding that is a subset of ASCII. The
//! unused ASCII range is used to encode half-width katakana characters. There
//! two modified ASCII characters: `0x5c` is converted to `0x00a5` (YEN SIGN)
//! and `0x7e` is converted to `0x203e` (OVERLINE). The rest of the are
//! undefined and not supported.

use std::borrow::Borrow;
use std::marker::PhantomData;

use crate::error::StringEncodingError::*;
use crate::helper::read_extension::StringReadSupport;
use crate::PicoriError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JisX0201Decoder<'x, I> {
    iter:    I,
    _marker: PhantomData<&'x ()>,
}

impl<I> JisX0201Decoder<'_, I> {
    fn new<'x>(iter: I) -> JisX0201Decoder<'x, I> {
        JisX0201Decoder {
            iter,
            _marker: PhantomData,
        }
    }

    pub fn decode_byte(byte: u8) -> Option<char> {
        match byte {
            // Modified ASCII character
            0x5c => Some('\u{00a5}'),
            0x7e => Some('\u{203e}'),
            // Unaltered ASCII character
            0x00..=0x7f => Some(byte as char),
            // Single-byte half-width katakana
            0xa1..=0xdf => {
                let unicode = 0xFF61 + (byte - 0xa1) as u32;
                char::from_u32(unicode)
            },
            _ => None,
        }
    }
}

impl<I> Iterator for JisX0201Decoder<'_, I>
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

pub struct JisX0201 {}

impl JisX0201 {
    pub fn iter<'iter, I>(iter: I) -> JisX0201Decoder<'iter, I>
    where
        I: Iterator + Clone,
        I::Item: Borrow<u8>,
    {
        JisX0201Decoder::new(iter)
    }

    pub fn all(data: &[u8]) -> Result<String, PicoriError> { Self::iter(data.iter()).collect() }

    pub fn first(data: &[u8]) -> Result<String, PicoriError> {
        Self::iter(data.iter())
            .take_while(|c| match c {
                Ok(c) => *c != 0 as char,
                Err(_) => false,
            })
            .collect()
    }
}

pub trait JisX0201Iterator: Iterator + Clone + Sized {
    fn jisx0201<'b>(self) -> JisX0201Decoder<'b, Self> { JisX0201Decoder::new(self) }
}

impl<I> JisX0201Iterator for I
where
    I: Iterator + Clone + Sized,
    I::Item: Borrow<u8>,
{
}

impl StringReadSupport for JisX0201 {
    fn read_string(data: &[u8]) -> Result<String, PicoriError> { Self::first(data) }
}
