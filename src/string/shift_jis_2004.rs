//! Shift JIS (Shift Japanese Industrial Standards) is an character encoding for
//! the Japanese language. It is built on the `JIS X 0201:1997` character set,
//! which is a single byte character encoding. The blank parts of `JIS X
//! 0201:1997` are used to encoded the `JIS X 0208:1997` (basic kanji) character
//! set, which is a double byte character encoding, .i.e, the first byte is
//! `0x81-0x9F` or `0xE0-0xEF`, and the second byte is `0x40-0x7E` or
//! `0x80-0xFC`. There are unallocated values in the first-byte range:
//! `0x85-0x87` and `0xEB-0xEF`. Similar the second byte has unallocated space,
//! only `94x2` characters can be used.
//!
//! There are multiple versions of Shift JIS. The two most common are the
//! `Shift JIS:1997` and `Shift JIS:2004`. `Shift JIS:1997` is the original
//! version using `JIS X 0208:1997`. `Shift JIS:2004` is the newer version,
//! which is using the new `JIS X 0213:2004` (extended kanji) character set.
//! `JIS X 0213:2004` is an extension of `JIS X 0208` with more supported kanji.
//! `Shift JIS:2004` is extended by using unallocated values in of first-byte,
//! .i.e, `0x85-0x87` and `0xEB-0xEF` are used and the first-byte range is
//! extended with `0xF0-0xFC`. `JIS X 0213:2004` includes characters that
//! Unicode can not present with a single code point. These characters are
//! encoded as a pair of code points.
//!
//! # Shift JIS vs JIS X 0208
//! Although `Shift JIS` uses `JIS X 0208`, they are not easily interchangeable.
//! A double-byte encoded character in `Shift JIS` is not equal to a double-byte
//! encoded character in `JIS X 0208`.
//!
//! # References
//! It has been hard to find a good reference for `Shift JIS`. The following are
//! the best references I have found:
//!
//! - [Shift JIS](https://en.wikipedia.org/wiki/Shift_JIS)
//! - [JIS X 0201](https://en.wikipedia.org/wiki/JIS_X_0201)
//! - [JIS X 0208](https://en.wikipedia.org/wiki/JIS_X_0208)
//! - [JIS X 0213](https://en.wikipedia.org/wiki/JIS_X_0213)
//! - [日本の文字コード](http://www.asahi-net.or.jp/~ax2s-kmtn/character/japan.html)
//! - [JIS基本漢字](http://www.asahi-net.or.jp/~ax2s-kmtn/ref/jisx0208.html)
//! - [JIS拡張漢字](http://www.asahi-net.or.jp/~ax2s-kmtn/ref/jisx0213/index.html)
//! - [Shift JIS Kanij Table](http://www.rikai.com/library/kanjitables/kanji_codes.sjis.shtml)

use std::borrow::Borrow;
use std::marker::PhantomData;

use crate::helper::DeserializableStringEncoding;
use crate::string::jis_x_0201::JisX0201Decoder;
use crate::StringEncodingError::*;
use crate::{ensure, PicoriError};

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

    fn decode_next(iter: &mut <I as IntoIterator>::IntoIter) -> Result<Next, PicoriError> {
        let byte = iter.next();
        if let Some(byte) = byte {
            let byte = *byte.borrow();
            if let Some(c) = JisX0201Decoder::<I>::decode_byte(byte) {
                return Ok(Next::One(c));
            }

            match byte {
                // First byte of a double-byte JIS X 0208 or JIS X 0213 character
                0x81..=0x9F | 0xE0..=0xFC => {
                    let next = iter.next().ok_or(UnexpectedEndOfInput)?;
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
    type Item = Result<char, PicoriError>;

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

pub struct ShiftJis2004 {}

impl ShiftJis2004 {
    pub fn iter<'iter, I>(iter: I) -> Decoder<'iter, I>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized,
    {
        Decoder::new(iter)
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
    fn sjis2004<'b>(self) -> Decoder<'b, Self> { Decoder::new(self) }
}

impl<I> IteratorExt for I
where
    I: IntoIterator,
    I::Item: Borrow<u8> + Sized,
{
}

impl DeserializableStringEncoding for ShiftJis2004 {
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
        let data = b"abc\x88\x9f\0def";
        assert_eq!(
            ShiftJis2004::deserialize_str(data).unwrap(),
            "abc亜".to_string()
        );
    }
}
