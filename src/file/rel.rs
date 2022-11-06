//! Deserialize and Serialize Dolphin Executables (`.dol` files).
//!
//! # Deserialize
//!
//! Deserialization can be done by calling [`parse`].
//!
//! ## Examples
//!
//! TODO: Add examples
//!
//! # Serialize
//!
//! TODO: Write this section.
use crate::error::DeserializeProblem::*;
use crate::helper::{ensure, Deserializer, Seeker};
use crate::Result;

pub struct Rel {
    /// The module number. Must be unique per `.rel` file.
    pub module:      u32,
    /// The offset of the name of the module. The name is available in the
    /// `framework.str`.
    pub name_offset: u32,
    /// The size of the name of the module.
    pub name_size:   u32,
    /// Sections.
    pub sections:    Vec<Section>,
    /// The prolog symbol.
    pub prolog:      Option<Symbol>,
    /// The epilog symbol.
    pub epilog:      Option<Symbol>,
    /// The unresolved symbol. This function is called when a symbol is not
    /// could not be resolved.
    pub unresolved:  Option<Symbol>,
}

pub struct Section {}

/// A symbol reference.
pub struct Symbol {
    /// Section where the symbol is located.
    pub section: u32,
    /// Offset of the symbol in the section.
    pub offset:  u32,
}

impl Rel {}

/// Deserialize [`Rel`] from a [`Deserializer`] + [`Seeker`].
///
/// # Panic
///
/// This function will not panic if the input is invalid. Any invalid input will
/// be returned as an error.
pub fn parse<D: Deserializer + Seeker>(reader: &mut D) -> Result<Rel> {
    let module = reader.deserialize_bu32()?;
    let _next = reader.deserialize_bu32()?; // should be 0, used at runtime
    let _prev = reader.deserialize_bu32()?; // should be 0, used at runtime
    let section_count = reader.deserialize_bu32()?;
    let section_offset = reader.deserialize_bu32()?;
    let name_offset = reader.deserialize_bu32()?;
    let name_size = reader.deserialize_bu32()?;
    let version = reader.deserialize_bu32()?;
    let _bss_size = reader.deserialize_bu32()?;
    let _relocation_offset = reader.deserialize_bu32()?;
    let _import_offset = reader.deserialize_bu32()?;
    let _import_size = reader.deserialize_bu32()?;
    let prolog_section = reader.deserialize_u8()?;
    let epilog_section = reader.deserialize_u8()?;
    let unresolved_section = reader.deserialize_u8()?;
    let _bss_section = reader.deserialize_u8()?; // should be 0, used at runtime
    let prolog_offset = reader.deserialize_bu32()?;
    let epilog_offset = reader.deserialize_bu32()?;
    let unresolved_offset = reader.deserialize_bu32()?;

    // version 2
    let (_align, _bss_align) = if version >= 2 {
        let align = reader.deserialize_bu32()?;
        let bss_align = reader.deserialize_bu32()?;
        (align, bss_align)
    } else {
        (1, 1)
    };

    // version 3
    let _fix_size = if version >= 3 {
        reader.deserialize_bu32()?
    } else {
        0
    };

    ensure!(version <= 3, UnsupportedVersion(version as usize));
    ensure!(section_count != 0, InvalidData("expected sections (got 0)"));
    ensure!(
        section_offset != 0,
        InvalidData("expected section offset (got 0)")
    );
    ensure!(
        name_offset != 0,
        InvalidData("expected name offset (got 0)")
    );
    ensure!(name_size != 0, InvalidData("expected name size (got 0)"));

    let prolog = if prolog_section != 0 {
        Some(Symbol {
            section: prolog_section as u32,
            offset:  prolog_offset,
        })
    } else {
        None
    };

    let epilog = if epilog_section != 0 {
        Some(Symbol {
            section: epilog_section as u32,
            offset:  epilog_offset,
        })
    } else {
        None
    };

    let unresolved = if unresolved_section != 0 {
        Some(Symbol {
            section: unresolved_section as u32,
            offset:  unresolved_offset,
        })
    } else {
        None
    };

    Ok(Rel {
        module,
        name_offset,
        name_size,
        sections: vec![],
        prolog,
        epilog,
        unresolved,
    })
}
