pub mod build;
pub mod compression;
pub mod decoding;
pub mod decompression;
pub mod encoding;
pub mod parse;

use std::panic::Location;

use super::{
    BuildProblem, CompressionProblem, DecodingProblem, DecompressionProblem, EncodingProblem,
    ParseProblem,
};

/// A error varient that can be used to represent any error that can occur in
/// this crate.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error that can occur when building.
    #[error("build: {0}")]
    Build(#[from] BuildProblem),

    /// An error that can occur when parsing.
    #[error("parse: {0}")]
    Parse(#[from] ParseProblem),

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

    /// Reading failed.
    #[error("read failed: {0} bytes ({1}) at {2}")]
    ReadFailed(usize, #[source] std::io::Error, &'static Location<'static>),

    /// Seeking failed.
    #[error("seek failed: {0} at {1}")]
    SeekFailed(#[source] std::io::Error, &'static Location<'static>),

    /// Unknown IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
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

pub(crate) use ensure;

pub trait ProblemLocation {
    #[track_caller]
    fn current() -> &'static std::panic::Location<'static> { std::panic::Location::caller() }
}

impl ProblemLocation for Location<'_> {}
