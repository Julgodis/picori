//! Deserialize and Serialize of various Nintendo file formats used by GameCube
//! and Wii games.
//!
//! Formats supported:
//! - [DOL - Dolphin Executable][`dol`]
//! - [REL - Relocatable Executable][`rel`]
//! - [GCM - GameCube Master Disc][`gcm`]
//! - [RARC - Nintendo RARC][`rarc`]
//! - [CISO - Compact ISO (WIB)][`ciso`]
//! - [ELF - Executable and Linkable Format][`elf`]

#[cfg(feature = "ciso")]
pub mod ciso;
#[cfg(feature = "dol")]
pub mod dol;
#[cfg(feature = "elf")]
pub mod elf;
#[cfg(feature = "rarc")]
pub mod rarc;
#[cfg(feature = "rel")]
pub mod rel;
#[cfg(feature = "gcm")]
pub mod gcm;
