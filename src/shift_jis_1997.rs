//! [`ShiftJis1997`] encoding.
//!
//! This module provides functionality to encode and decode text in the `Shift
//! JIS` (Shift Japanese Industrial Standards) version `1997`. There are
//! multiple versions of [Shift JIS][`ShiftJis1997`], but this module
//! only supports the version known as [Shift JIS 1997][`ShiftJis1997`].
//!
//! [Shift JIS][`ShiftJis1997`] is an encoding for Japanese characters and a
//! extension to the [JIS X 0201][`crate::JisX0201`] encoding. The
//! first 128 characters and the half-width katakana section of [Shift
//! JIS][`ShiftJis1997`] are identical to [JIS X
//! 0201][`crate::JisX0201`]. The remaining unused characters in [JIS
//! X 0201][`crate::JisX0201`] are taken advantage of to encode more
//! characters and kanji. Specifically, the range `[0x81,0x9F]` and
//! `[0xE0,0xEF]`. Limited to single-byte encoding is insufficient to encompass
//! a large set of Japanese characters (more than 47 characters will be
//! required). To solve this problem, [Shift JIS][`ShiftJis1997`] uses a
//! two-byte encoding scheme. The first byte (lead-byte) is in the ranges
//! described above, i.e., `[0x81,0x9F]` or `[0xE0,0xEF]`. The second byte takes
//! any value in the range `[0x40,0xFC]`, excluding the specific value of
//! `0x7f`. For a total of `8,789` (`47x187`) characters to be encoded with
//! spaces for further expansion in the future.
//!
//! The character set that [Shift JIS][`ShiftJis1997`] uses is defined by `JIS X
//! 0208`. `JIS X 0208` is another two-byte encoding specified by JIS containing
//! 6897 characters with a purpose of 情報交換 (information interchange). The
//! "Shift" in [Shift JIS][`ShiftJis1997`] refers to the fact that the first
//! byte, in the two-byte encoding, is shifted around half-width katakana.
//!
//! There are four standards of `JIS X 0208`: `JIS C 6226-1978`, `JIS C
//! 6226-1983`, `JIS X 0208-1990` (`90JIS`), and `JIS X 0208-1997` (`97JIS`).
//! [Shift JIS 1997][`ShiftJis1997`] uses the fourth standard released in the
//! same year. Since 1997 there have been no new releases of `JIS X 0208`.
//! Instead, a new specification was released, `JIS X 0213`, which extends `JIS
//! X 0208` with more characters. [Shift JIS 1997][`ShiftJis1997`] does not use
//! `JIS X 0213`.
//!
//! # Examples
//! TODO: Add examples
//!
//! # References
//! Finding references that were still available was incredible difficult. Both
//! for [Shift JIS][`ShiftJis1997`] encoding and the related ones.
//!
//! - [Shift JIS](https://en.wikipedia.org/wiki/Shift_JIS)
//! - [JIS X 0201](https://en.wikipedia.org/wiki/JIS_X_0201)
//! - [JIS X 0208](https://en.wikipedia.org/wiki/JIS_X_0208)
//! - [日本の文字コード](http://www.asahi-net.or.jp/~ax2s-kmtn/character/japan.html)
//! - [JIS基本漢字](http://www.asahi-net.or.jp/~ax2s-kmtn/ref/jisx0208.html)
//! - [Shift JIS Kanji Table](http://www.rikai.com/library/kanjitables/kanji_codes.sjis.shtml)
//! - [JIS X 0213 Code Mapping Tables](http://x0213.org/codetable/index.en.html)

use std::borrow::Borrow;
use std::io::{BufReader, Read};
use std::marker::PhantomData;
use std::panic::Location;

use crate::error::DecodingProblem::*;
use crate::helper::{ensure, ParseStringEncoding, ProblemLocation, Parser};
use crate::jis_x_0201::Decoder as JisX0201Decoder;
use crate::Result;

mod internal {
    include!(concat!(env!("OUT_DIR"), "/shift_jis_1997.rs"));
}

/// [`ShiftJis1997`] encoding.
pub struct ShiftJis1997 {}

enum Next {
    EndOfInput,
    One(char),
}

/// A iterator decoder for the [`ShiftJis1997`] encoding.
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

    fn decode_next(iter: &mut <I as IntoIterator>::IntoIter) -> Result<Next> {
        let byte = iter.next();
        if let Some(byte) = byte {
            let byte = *byte.borrow();
            if let Some(c) = JisX0201Decoder::<I>::decode_byte(byte) {
                return Ok(Next::One(c));
            }

            match byte {
                // First byte of a double-byte JIS X 0208 character
                0x81..=0x9F | 0xE0..=0xFC => {
                    let next = iter
                        .next()
                        .ok_or_else(|| UnexpectedEndOfData(Location::current()))?;
                    let next = *next.borrow();
                    let (first, last, offset) = internal::SJIS_1997_UTF8_T[byte as usize];
                    ensure!(
                        next >= first && next <= last,
                        InvalidByte(next, Location::current())
                    );
                    let relative = (next - first) as usize;
                    let index = offset + relative;
                    let value = internal::SJIS_1997_UTF8_S[index];
                    ensure!(value != 0, InvalidByte(next, Location::current()));
                    ensure!(
                        (value & 0x8000_0000) == 0,
                        InvalidByte(next, Location::current())
                    );
                    Ok(Next::One(unsafe { char::from_u32_unchecked(value) }))
                },
                // Invalid as first byte
                _ => Err(InvalidByte(byte, Location::current()).into()),
            }
        } else {
            Ok(Next::EndOfInput)
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
        match Self::decode_next(&mut self.iter) {
            Ok(Next::EndOfInput) => None,
            Ok(Next::One(c)) => Some(Ok(c)),
            Err(e) => Some(Err(e)),
        }
    }
}

impl ShiftJis1997 {
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
/// [`IteratorExt::sjis1997`] for decoding as [Shift JIS 1997][`ShiftJis1997`]
/// strings.
pub trait IteratorExt
where
    Self: IntoIterator + Sized,
    Self::Item: Borrow<u8> + Sized,
{
    /// Decode self iterator of bytes as [Shift JIS 1997][`ShiftJis1997`].
    fn sjis1997<'b>(self) -> Decoder<'b, Self> { Decoder::new(self) }
}

impl<I> IteratorExt for I
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
}

impl ParseStringEncoding for ShiftJis1997 {
    fn parse_str<I>(iter: I) -> Result<String>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        Self::first(iter)
    }

    fn write_str(_data: &str, _buffer: &mut [u8]) -> Result<usize> {
        todo!("ShiftJis1997::write_str")
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
        let data = b"abc\x88\x9f\0def";
        assert_eq!(ShiftJis1997::parse_str(data).unwrap(), "abc亜".to_string());
    }
}
