//! `JIS X 0201` (ANK) is a single-byte encoding that is a subset of ASCII. The
//! unused ASCII range is used to encode half-width katakana characters. There
//! two modified ASCII characters: `0x5c` is converted to `0x00a5` (YEN SIGN)
//! and `0x7e` is converted to `0x203e` (OVERLINE). The rest of the are
//! undefined and not supported.

use std::borrow::Borrow;
use std::marker::PhantomData;

use crate::helper::error::StringEncodingError::*;
use crate::helper::DeserializableStringEncoding;
use crate::PicoriError;

pub struct JisX0201Decoder<'x, I>
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
    iter:    <I as IntoIterator>::IntoIter,
    _marker: PhantomData<&'x ()>,
}

impl<I> JisX0201Decoder<'_, I>
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
    fn new<'x>(iter: I) -> JisX0201Decoder<'x, I> {
        JisX0201Decoder {
            iter:    iter.into_iter(),
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
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
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
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        JisX0201Decoder::new(iter)
    }

    pub fn all<I>(iter: I) -> Result<String, PicoriError>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        Self::iter(iter).collect()
    }

    pub fn first<I>(iter: I) -> Result<String, PicoriError>
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
    fn jisx0201<'b>(self) -> JisX0201Decoder<'b, Self> { JisX0201Decoder::new(self) }
}

impl<I> IteratorExt for I
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
}

impl DeserializableStringEncoding for JisX0201 {
    fn deserialize_str<I>(iter: I) -> Result<String, PicoriError>
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
        assert_eq!(JisX0201::deserialize_str(data).unwrap(), "abc".to_string());
    }
}
