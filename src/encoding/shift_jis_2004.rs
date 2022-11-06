use std::borrow::Borrow;
use std::marker::PhantomData;

use crate::encoding::jis_x_0201::Decoder as JisX0201Decoder;
use crate::error::DecodingProblem::*;
use crate::helper::{ensure, DeserializableStringEncoding};
use crate::Result;

/// This module provides functionality to encode and decode text in the [Shift
/// JIS][`ShiftJis2004`] (Shift Japanese Industrial Standards) version `2004`.
///
/// [Shift JIS 2004][`ShiftJis2004`] is an updated version of [Shift JIS
/// 1997][`crate::encoding::ShiftJis1997`]. It includes the uses
/// of `JIS X 0213-2000` and `JIS X 0213-2004`. The major differences is that
/// [Shift JIS 2004][`ShiftJis2004`] has more lead-byte that are available and
/// new characters introduced.
///
/// For information on how [Shift JIS][`ShiftJis2004`] encoding works see [Shift
/// JIS 1997][`crate::encoding::ShiftJis1997`] and the references that are
/// linked below.
///
/// # Examples
/// TODO: Add examples
///
/// # References
/// Finding references that were still available was incrdible difficult. Both
/// for [Shift JIS][`ShiftJis2004`] encoding and the related ones.
///
/// - [Shift JIS](https://en.wikipedia.org/wiki/Shift_JIS)
/// - [JIS X 0201](https://en.wikipedia.org/wiki/JIS_X_0201)
/// - [JIS X 0213](https://en.wikipedia.org/wiki/JIS_X_0213)
/// - [日本の文字コード](http://www.asahi-net.or.jp/~ax2s-kmtn/character/japan.html)
/// - [JIS拡張漢字](http://www.asahi-net.or.jp/~ax2s-kmtn/ref/jisx0213/index.html)
/// - [JIS X 0213 Code Mapping Tables](http://x0213.org/codetable/index.en.html)
/// - [Shift JIS Kanji Table](http://www.rikai.com/library/kanjitables/kanji_codes.sjis.shtml)
pub struct ShiftJis2004 {}

mod internal {
    include!(concat!(env!("OUT_DIR"), "/shift_jis_2004.rs"));
}

pub enum Next {
    EndOfInput,
    One(char),
    Two(char, char),
}

pub struct Decoder<'x, I>
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
    iter:     <I as IntoIterator>::IntoIter,
    buffered: Option<char>,
    _marker:  PhantomData<&'x ()>,
}

impl<I> Decoder<'_, I>
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
    fn new<'x>(iter: I) -> Decoder<'x, I> {
        Decoder {
            iter:     iter.into_iter(),
            buffered: None,
            _marker:  PhantomData,
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
                // First byte of a double-byte JIS X 0208 or JIS X 0213 character
                0x81..=0x9F | 0xE0..=0xFC => {
                    let next = iter.next().ok_or(UnexpectedEndOfData)?;
                    let next = *next.borrow();
                    let (first, last, offset) = internal::SJIS_2004_UTF8_T[byte as usize];
                    ensure!(next >= first && next <= last, InvalidByte(next));
                    let relative = (next - first) as usize;
                    let index = offset + relative;
                    let value = internal::SJIS_2004_UTF8_S[index];
                    ensure!(value != 0, InvalidByte(next));
                    if value & 0x8000_0000 != 0 {
                        let index = (value & 0x7fff_ffff) as usize;
                        let (first, second) = internal::SJIS_2004_UTF8_D[index];
                        Ok(Next::Two(
                            unsafe { char::from_u32_unchecked(first) },
                            unsafe { char::from_u32_unchecked(second) },
                        ))
                    } else {
                        Ok(Next::One(unsafe { char::from_u32_unchecked(value) }))
                    }
                },
                // Invalid as first byte
                _ => Err(InvalidByte(byte).into()),
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
        if let Some(value) = self.buffered {
            self.buffered = None;
            Some(Ok(value))
        } else {
            match Self::decode_next(&mut self.iter) {
                Ok(Next::EndOfInput) => None,
                Ok(Next::One(c)) => Some(Ok(c)),
                Ok(Next::Two(first, second)) => {
                    self.buffered = Some(second);
                    Some(Ok(first))
                },
                Err(e) => Some(Err(e)),
            }
        }
    }
}

impl ShiftJis2004 {
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
/// [`IteratorExt::sjis2004`] for decoding as [Shift JIS 2004][`ShiftJis2004`]
/// strings.
pub trait IteratorExt
where
    Self: IntoIterator + Sized,
    Self::Item: Borrow<u8> + Sized,
{
    /// Decode self iterator of bytes as [Shift JIS 2004][`ShiftJis2004`].
    fn sjis2004<'b>(self) -> Decoder<'b, Self> { Decoder::new(self) }
}

impl<I> IteratorExt for I
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
}

impl DeserializableStringEncoding for ShiftJis2004 {
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
        let data = b"abc\x88\x9f\0def";
        assert_eq!(
            ShiftJis2004::deserialize_str(data).unwrap(),
            "abc亜".to_string()
        );
    }
}
