//! [`JisX0201`] encoding.
//!
//! [JIS X 0201][`JisX0201`] (ANK) is a single-byte encoding specified by JIS
//! (Japanese Industrial Standards) and is built upon the 7-bit
//! [ASCII][`crate::Ascii`] encoding. The first 7-bit are untouched
//! except for two characters. The [ASCII][`crate::Ascii`] character
//! `0x5C` (Reverse Solidus) replaced by the Unicode character 'U+00A5' (Yen
//! Sign) and the [ASCII][`crate::Ascii`] character `0x7E` (Tilde)
//! replaced by the Unicode character `U+203E` (Overline). The eighth bit
//! provide space for the phonetic Japanese katakana signs in half-width style.
//!
//! [JIS X 0201][`JisX0201`] is encoding that [Shift
//! JIS][`crate::ShiftJis1997`] is based upon.

use std::borrow::Borrow;
use std::io::{BufReader, Read};
use std::marker::PhantomData;
use std::panic::Location;

use crate::error::DecodingProblem::*;
use crate::helper::{ParseStringEncoding, ProblemLocation, Parser};
use crate::Result;

/// [`JisX0201`] encoding.
pub struct JisX0201 {}

/// A iterator decoder for the [`JisX0201`] encoding.
pub struct Decoder<'x, I>
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
    iter:    <I as IntoIterator>::IntoIter,
    _marker: PhantomData<&'x ()>,
}

impl<I> Decoder<'_, I>
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
    fn new<'x>(iter: I) -> Decoder<'x, I> {
        Decoder {
            iter:    iter.into_iter(),
            _marker: PhantomData,
        }
    }

    /// Decode a single byte.
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

impl<I> Iterator for Decoder<'_, I>
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
                None => Err(InvalidByte(*byte, Location::current()).into()),
            })
        } else {
            None
        }
    }
}

impl JisX0201 {
    /// Create an iterator that decodes the given iterator of bytes into
    /// characters.
    pub fn iter<'iter, I>(iter: I) -> Decoder<'iter, I>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        Decoder::new(iter)
    }

    /// Decode all bytes into a string. Will continue passed NULL bytes and only
    /// stop at the end of the iterator or if an decoding error occurs.
    pub fn all<I>(iter: I) -> Result<String>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        Self::iter(iter).collect()
    }

    /// Decode the first string (until a NULL character is reached) from the
    /// given iterator.
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

/// Extension trait for iterators of bytes and adds the helper function
/// [`IteratorExt::jisx0201`] for decoding as [JIS X 0201][`JisX0201`] strings.
pub trait IteratorExt
where
    Self: IntoIterator + Sized,
    Self::Item: Borrow<u8> + Sized,
{
    /// Decode self iterator of bytes as [JIS X 0201][`JisX0201`].
    fn jisx0201<'b>(self) -> Decoder<'b, Self> { Decoder::new(self) }
}

impl<I> IteratorExt for I
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
}

impl ParseStringEncoding for JisX0201 {
    fn parse_str<I>(iter: I) -> Result<String>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        Self::first(iter)
    }

    fn write_str(_data: &str, _buffer: &mut [u8]) -> Result<usize> {
        todo!("JisX0201::write_str")
    }

    fn from_binary(reader: &mut impl Parser) -> Result<String> {
        let buffer = BufReader::new(reader);
        let iter = buffer
            .bytes()
            .take_while(|x| x.is_ok())
            .filter_map(|x| match x {
                Ok(x) => Some(x),
                Err(_) => None,
            });
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
    fn parse_str() {
        let data = b"abc\0def";
        assert_eq!(JisX0201::parse_str(data).unwrap(), "abc".to_string());
    }
}
