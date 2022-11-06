#![doc(
    html_logo_url = "https://raw.githubusercontent.com/Julgodis/picori/master/assets/images/picori_logo_512.png"
)]
#![doc = include_str!("../README.md")]
// TODO: Is there any performance benefit of allowing uninit vec unsafe code?
#![allow(clippy::uninit_vec)]
// TODO: deny missing_docs once all public items have docs
#![allow(missing_docs)]
#![warn(warnings)]

#[cfg(feature = "compression")]
pub mod compression;
#[cfg(feature = "demangle")]
pub mod demangle;
#[cfg(feature = "encoding")]
pub mod encoding;
#[cfg(feature = "file")]
pub mod file;

pub use helper::{Deserializer, Error, Reader, Result, Seeker};

mod helper;

/// The [Error][`Error`] enum can indicate a variety of errors that can occur.
/// This module contains the error types that can be returned by this library.
pub mod error {
    pub use super::helper::{
        CompressionProblem, DecodingProblem, DecompressionProblem, DeserializeProblem,
        EncodingProblem, SerializeProblem,
    };
}
