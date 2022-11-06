//! TODO:

pub mod ascii;
pub mod jis_x_0201;
pub mod shift_jis;

use crate::error::PicoriError;

/// A trait for any string decoder that can take a byte iterator or a byte slice
/// and decode it into a `String` (UTF-8 string).
pub trait StringDecoder {
    fn decode_iterator<T>(input: T) -> Result<String, PicoriError>
    where
        T: Iterator<Item = u8>;

    fn decode_bytes(input: &[u8]) -> Result<String, PicoriError> {
        Self::decode_iterator(input.iter().copied())
    }

    fn decode_until_zero_iterator<T>(input: T) -> Result<String, PicoriError>
    where
        T: Iterator<Item = u8>;

    fn decode_until_zero(input: &[u8]) -> Result<String, PicoriError> {
        Self::decode_until_zero_iterator(input.iter().copied())
    }
}
