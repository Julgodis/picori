//! Picori is a library for building modding and decompliation tools for
//! GameCube and Wii games. It includes support encoding and decoding many of
//! Nintendos file formats, support for common compression algorithms, and
//! support to demangle C++ symbols.
//!
//! ## Formats
//!
//! Picori supports the following formats:
//!
//! - DOL - Dolphin Executable
//! - REL - Relocatable Executable
//! - GCM - GameCube Master Disc
//! - RARC - Nintendo RARC
//! - ELF - Executable and Linkable Format[^note-elf]
//!
//! In the future adding support for more formats is planned.
//!
//! [^note-elf]: ELF is not a specific format used by either GameCube or Wii,
//! but no known compiler outputs DOL files direct (and for good reasons),
//! instead they produce ELF files. Support for   ELF (specific to GameCube and
//! Wii) are useful.
//!
//! ## Compression
//!
//! Picori supports the following compression algorithms:
//!
//! - Yaz0
//! - Yay0
//!
//! ## C++ Demangler
//!
//! Picori also includes a C++ demangler for the compiler used to compile
//! GameCube games. The only compiler that is supported at the moment is MWCC
//! (etrowerks CodeWarrior Compiler).
//! 

#![feature(try_trait_v2)]
#![feature(maybe_uninit_uninit_array)]
#![feature(read_buf)]

pub mod format;
pub mod compression;
pub mod demangle;
pub mod string;

mod error;
mod helper;
mod stream;
mod endian;

pub use self::stream::Deserializeble;
pub use self::stream::DeserializeError;

pub use self::helper::SliceReader;