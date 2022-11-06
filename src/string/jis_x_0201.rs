//! `JIS X 0201` (ANK) is a single-byte encoding that is a subset of ASCII. The
//! unused ASCII range is used to encode half-width katakana characters. There
//! two modified ASCII characters: `0x5c` is converted to `0x00a5` (YEN SIGN)
//! and `0x7e` is converted to `0x203e` (OVERLINE). The rest of the are
//! undefined and not supported.

use super::StringDecoder;
use crate::error::StringEncodingError::*;
use crate::PicoriError;

pub struct JisX0210Decoder {}

impl JisX0210Decoder {
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

impl StringDecoder for JisX0210Decoder {
    fn decode_iterator<T>(input: T) -> Result<String, PicoriError>
    where
        T: Iterator<Item = u8>,
    {
        let mut output = String::new();
        let mut iter = input.peekable();

        while let Some(byte) = iter.next() {
            match Self::decode_byte(byte) {
                Some(c) => output.push(c),
                None => return Err(InvalidByte(byte).into()),
            }
        }

        Ok(output)
    }

    fn decode_until_zero_iterator<T>(input: T) -> Result<String, PicoriError>
    where
        T: Iterator<Item = u8>,
    {
        Self::decode_iterator(input.take_while(|b| *b != 0))
    }
}
