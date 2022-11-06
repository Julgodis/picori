pub mod ascii;
pub mod shift_jis;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StringError {
    #[error("unable to decode to utf-8")]
    UnableToDecode,

    #[error("utf8 error")]
    Utf8Error(#[from] std::str::Utf8Error),
}

impl StringError {
    pub fn new<X>(other: X) -> Self
    where
        X: Into<StringError>,
    {
        return other.into();
    }
}

pub trait StringEncoding {
    fn decode_bytes(input: &[u8]) -> Result<String, StringError>;
}
