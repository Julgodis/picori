
pub mod parse;
pub mod build;

/// Enum for possible compression problems that can occur.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum CompressionProblem {}

/// Enum for possible decompression problems that can occur.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum DecompressionProblem {
    /// Reading decompression header failed.
    #[error("invalid header")]
    InvalidHeader(&'static str),

    /// Decompressing data failed.
    #[error("invalid data")]
    InvalidData(&'static str),

    /// Decompression failed because of an unexpected end of data.
    #[error("unexpected EOD")]
    UnexpectedEndOfData,

    /// Decompression failed because of an unexpected destination size.
    #[error("invalid decompression size")]
    InvalidDecompressedSize,
}

/// Enum for possible encoding problems that can occur.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum EncodingProblem {}

/// Enum for possible decoding problems that can occur.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum DecodingProblem {
    /// Decoding try to create a character that is not valid in UTF-8.
    #[error("invalid code point: {0}")]
    InvalidCodePoint(usize),

    /// Invalid byte sequence.
    #[error("invalid byte: {0}")]
    InvalidByte(u8),

    /// Decoding failed due to unexpected end of data, .i.e, it requires more
    /// data to fully decode.
    #[error("unexpected EOD")]
    UnexpectedEndOfData,
}

/// A error varient that can be used to represent any error that can occur in
/// this crate. It include wrappers for `std::io::Error` and
/// `std::str::Utf8Error`.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error that can occur when building.
    #[error("build: {0}")]
    Serialize(#[from] BuildProblem),

    /// An error that can occur when parsing.
    #[error("parse: {0}")]
    Deserialize(#[from] ParseProblem),

    /// An error that can occur when compressing.
    #[error("compression: {0}")]
    Compression(#[from] CompressionProblem),

    /// An error that can occur when decompressing.
    #[error("decompression: {0}")]
    Decompression(#[from] DecompressionProblem),

    /// An error that can occur when encoding.
    #[error("encoding: {0}")]
    Encoding(#[from] EncodingProblem),

    /// An error that can occur when decoding.
    #[error("decoding: {0}")]
    Decoding(#[from] DecodingProblem),

    /// An wrapped error from `std::io`.
    #[error("io error")]
    IoError(#[from] std::io::Error),

    /// An wrapped error from `std::str`.
    #[error("utf8 error")]
    Utf8Error(#[from] std::str::Utf8Error),
}

/// A specialized [`Result`] type for Picori. This type is broadly used across
/// internal and public APIs. The Err variant is [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}

use std::panic::Location;

pub(crate) use ensure;

use super::{ParseProblem, BuildProblem};


pub trait ProblemLocation {
    #[track_caller]
    fn current() -> &'static std::panic::Location<'static> {
        std::panic::Location::caller()
    }
}

impl ProblemLocation for Location<'_> {}