//! [ASCII][`Ascii`] encoding.
//! 
//! [ASCII][`Ascii`] is a 7-bit encoding designed for information interchange in
//! English. Bytes with the eighth bit set are considered invalid and will cause
//! an [`crate::error::DecodingProblem::InvalidByte`] to be returned.
//!
//! # Examples
//! TODO: Add examples

use std::borrow::Borrow;
use std::marker::PhantomData;

use crate::error::DecodingProblem::*;
use crate::Result;
use crate::helper::ParseStringEncoding;

/// [ASCII][`Ascii`] encoding.
pub struct Ascii {}

/// A iterator decoder for the [`Ascii`] encoding.
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

    fn decode_byte(byte: u8) -> Option<char> {
        match byte {
            // ASCII character
            0x00..=0x7f => Some(byte as char),
            // Invalid
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
                None => Err(InvalidByte(*byte).into()),
            })
        } else {
            None
        }
    }
}

impl Ascii {
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
/// [`IteratorExt::ascii`] for decoding as [ASCII][`Ascii`] strings.
pub trait IteratorExt
where
    Self: IntoIterator + Sized,
    Self::Item: Borrow<u8> + Sized,
{
    /// Decode self iterator of bytes as [ASCII][`Ascii`].
    fn ascii<'b>(self) -> Decoder<'b, Self> { Decoder::new(self) }
}

impl<I> IteratorExt for I
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
}

impl ParseStringEncoding for Ascii {
    fn parse_str<I>(iter: I) -> Result<String>
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
    fn parse_str() {
        let data = b"abc\0def";
        assert_eq!(Ascii::parse_str(data).unwrap(), "abc".to_string());
    }
}
