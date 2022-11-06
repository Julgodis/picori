//!
//! TODO:
//! 
//! 

pub mod ascii;
pub mod shift_jis;

use crate::error::PicoriError;

pub trait StringEncoding {
    fn decode_bytes(input: &[u8]) -> Result<String, PicoriError>;
}
