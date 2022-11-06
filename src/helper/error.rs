#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum SerializeProblem {}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum DeserializeProblem {
    #[error("invalid header: {0}")]
    InvalidHeader(&'static str),

    #[error("invalid data: {0}")]
    InvalidData(&'static str),

    #[error("unsupported version: {0}")]
    UnsupportedVersion(usize),
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum CompressionProblem {}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum DecompressionProblem {
    #[error("invalid header")]
    InvalidHeader(&'static str),

    #[error("invalid data")]
    InvalidData(&'static str),

    #[error("unexpected EOF")]
    UnexpectedEndOfInput,
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum EncodingProblem {}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum DecodingProblem {
    #[error("invalid code point: {0}")]
    InvalidCodePoint(usize),

    #[error("invalid byte: {0}")]
    InvalidByte(u8),

    #[error("unexpected EOF")]
    UnexpectedEndOfInput,
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("integer overflow error")]
    IntegerOverflow(),

    #[error("serialize: {0}")]
    Serialize(#[from] SerializeProblem),

    #[error("deserialize: {0}")]
    Deserialize(#[from] DeserializeProblem),

    #[error("compression: {0}")]
    Compression(#[from] CompressionProblem),

    #[error("decompression: {0}")]
    Decompression(#[from] DecompressionProblem),

    #[error("encoding: {0}")]
    Encoding(#[from] EncodingProblem),

    #[error("decoding: {0}")]
    Decoding(#[from] DecodingProblem),

    #[error("io error")]
    IoError(#[from] std::io::Error),

    #[error("utf8 error")]
    Utf8Error(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}

pub(crate) use ensure;
