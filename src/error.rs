use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum FormatError {
    #[error("invalid header: {0}")]
    InvalidHeader(&'static str),

    #[error("invalid data: {0}")]
    InvalidData(&'static str),
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CompressionError {
    #[error("invalid header")]
    InvalidHeader(),

    #[error("invalid data")]
    InvalidData(),
}

#[derive(Error, Debug)]
pub enum StringEncodingError {
    #[error("unable to decode: {0}")]
    UnableToDecode(&'static str),

    #[error("unable to encode: {0}")]
    UnableToEncode(&'static str),

    #[error("invalid codepoint: {0}")]
    InvalidCodepoint(usize),

    #[error("missing data: {0}")]
    MissingData(&'static str),
}

impl StringEncodingError {
    pub fn new<X>(other: X) -> Self
    where
        X: Into<StringEncodingError>,
    {
        other.into()
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum PicoriError {
    #[error("integer overflow error")]
    IntegerOverflow(),

    #[error("format error: {0}")]
    Format(#[from] FormatError),

    #[error("compression error: {0}")]
    Compression(#[from] CompressionError),

    #[error("string encoding error: {0}")]
    StringEncodingError(#[from] StringEncodingError),

    #[error("io error")]
    IoError(#[from] std::io::Error),

    #[error("utf8 error")]
    Utf8Error(#[from] std::str::Utf8Error),
}

impl PicoriError {
    pub fn new<T>(error: T) -> Self
    where
        T: Into<PicoriError>,
    {
        error.into()
    }
}



macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}

pub(crate) use ensure;