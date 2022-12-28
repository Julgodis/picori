use std::panic::Location;

/// Enum for possible encoding problems that can occur.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum EncodingProblem {
    #[error("buffer too small: {0}")]
    BufferTooSmall(&'static Location<'static>),

    #[error("unable to encode code point: {0} at {1}")]
    UnableToEncodeCodePoint(char, &'static Location<'static>),
}
