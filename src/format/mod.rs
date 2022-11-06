//! Deserialize and Serialize of various Nintendo file formats used by GameCube and Wii games.
//! 
//! Formats supported:
//! - [DOL - Dolphin Executable][`dol`]
//! - [REL - Relocatable Executable][`rel`]
//! - [GCM - GameCube Master Disc][`gcm`]
//! - [RARC - Nintendo RARC][`rarc`]
//! - [CISO - Compact ISO (WIB])[`ciso`]
//! - [ELF - Executable and Linkable Format][`elf`]

pub mod dol;
pub mod gcm;
pub mod ciso;
