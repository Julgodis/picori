//! Parse Relocatable module (`.rel`).
//!
//! # Parse
//!
//! Parse from binary stream by calling [`Rel::from_binary`]. On error a
//! [Error][`crate::Error`] is return. Otherwise, the parsing succeeded and you
//! get back a [`Rel`] struct.
//!
//! ## Example
//!
//! This is an example of how to parse a `.rel` file.
//!
//! ```no_run
//! # use std::fs::File;
//! # use picori::Result;
//! fn main() -> Result<()> {
//!     let mut file = File::open("module.rel")?;
//!     let _ = picori::Rel::from_binary(&mut file)?;
//!     Ok(())
//! }
//! ```

use crate::error::ParseProblem;
use crate::helper::{ensure, Parser, ProblemLocation, Seeker};
use crate::Result;

/// `.rel` file object.
#[derive(Debug, Clone)]
pub struct Rel {
    /// The module number. Must be unique per `.rel` file.
    pub module: u32,
    /// The version number of the `.rel` file.
    pub version: u32,
    /// The offset of the name of the module. The name is available in the
    /// `framework.str`.
    pub name_offset: u32,
    /// The size of the name of the module.
    pub name_size: u32,
    /// Sections.
    pub sections: Vec<Section>,
    /// Import tables.
    pub import_tables: Vec<ImportTable>,
    /// The prolog symbol.
    pub prolog: Option<Symbol>,
    /// The epilog symbol.
    pub epilog: Option<Symbol>,
    /// The unresolved symbol. This function is called when a symbol is not
    /// could not be resolved.
    pub unresolved: Option<Symbol>,
    /// Section alignment.
    pub alignment: u32,
    /// `.bss` section alignment.
    pub bss_alignment: u32,
    /// `parse`: Unknown.
    pub fix_size: u32,
    /// `parse`: Relocation offset.
    pub relocation_offset: Option<u32>,
    /// `parse`: Import offset.
    pub import_offset: Option<u32>,
    /// `parse`: Import size.
    pub import_size: Option<u32>,
}

/// Relocatable module section.
#[derive(Debug, Clone, Default)]
pub struct Section {
    /// Offset in the `.rel` file.
    pub offset:     u32,
    /// Size of the section.
    pub size:       u32,
    /// Executable flag.
    pub executable: bool,
    /// Unknown flag.
    pub unknown:    bool,
    /// Section data.
    pub data:       Vec<u8>,
}

/// Import kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportKind {
    /// No-op import.
    None,
    /// R_PPC_ADDR32 relocation (`S + A`).
    Addr32,
    /// R_PPC_ADDR24 relocation (`(S + A) >> 2`).
    Addr24,
    /// R_PPC_ADDR16 relocation (`S + A`).
    Addr16,
    /// R_PPC_ADDR16_LO relocation (`#lo(S + A)`).
    Addr16Lo,
    /// R_PPC_ADDR16_HI relocation (`#hi(S + A)`).
    Addr16Hi,
    /// R_PPC_ADDR16_HA relocation (`#ha(S + A)`).
    Addr16Ha,
    /// R_PPC_ADDR14 relocation (`(S + A) >> 2`).
    Addr14,
    /// R_PPC_REL24 relocation (`(S + A - P) >> 2`).
    Rel24,
    /// R_PPC_REL14 relocation (`(S + A - P) >> 2`).
    Rel14,
    /// Dolphin-specific relocation, used to support long offset values.
    DolphinNop,
    /// Dolphin-specific relocation, used to set target section.
    DolphinSection,
    /// Dolphin-specific relocation, indicates end of import table.
    DolphinEnd,
    /// Dolphin-specific relocation, unknown purpose.
    DolphinMRKREF,
}

/// Import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Import {
    /// Kind of the import.
    pub kind:    ImportKind,
    /// Section of imported symbol.
    pub section: u8,
    /// Offset of imported symbol.
    pub offset:  u16,
    /// Import addend.
    pub addend:  u32,
}

/// Import table.
#[derive(Debug, Clone)]
pub struct ImportTable {
    /// Import table for module.
    pub module:  u32,
    /// Offset to import table in the `.rel` file.
    pub offset:  u32,
    /// List of imports.
    pub imports: Vec<Import>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A symbol reference.
pub struct Symbol {
    /// Section where the symbol is located.
    pub section: u32,
    /// Offset of the symbol in the section.
    pub offset:  u32,
}

/// Reference to section and offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionOffset {
    /// Section.
    pub section: u32,
    /// Offset in the section.
    pub offset:  u32,
}

/// Relocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Relocation {
    /// Relocation kind. This is the same as the import kind. But `Dolphin*` are
    /// not used for relocations.
    pub kind:      ImportKind,
    /// Module of the referenced symbol.
    pub module:    u32,
    /// Reference to the symbol.
    pub reference: SectionOffset,
    /// Where the relocation is located (where the location of the bytes that
    /// should be modified are).
    pub target:    SectionOffset,
}

impl Rel {
    /// Parse [`Rel`] from binary stream.
    ///
    /// This function _should_ not panic and if any error occurs, it will return
    /// [`Err`] of type [`Error`][`crate::Error`]/[`ParseProblem`].
    pub fn from_binary<D: Parser + Seeker>(mut reader: D) -> Result<Self> {
        let base = reader.position()?;
        let module = reader.bu32()?;
        let _next = reader.bu32()?; // should be 0, used at runtime
        let _prev = reader.bu32()?; // should be 0, used at runtime
        let section_count = reader.bu32()?;
        let section_offset = reader.bu32()?;
        let name_offset = reader.bu32()?;
        let name_size = reader.bu32()?;
        let version = reader.bu32()?;
        let _bss_size = reader.bu32()?;
        let relocation_offset = reader.bu32()?;
        let import_offset = reader.bu32()?;
        let import_size = reader.bu32()?;
        let prolog_section = reader.u8()?;
        let epilog_section = reader.u8()?;
        let unresolved_section = reader.u8()?;
        let _bss_section = reader.u8()?; // should be 0, used at runtime
        let prolog_offset = reader.bu32()?;
        let epilog_offset = reader.bu32()?;
        let unresolved_offset = reader.bu32()?;

        // version 2
        let (align, bss_align) = if version >= 2 {
            let align = reader.bu32()?;
            let bss_align = reader.bu32()?;
            (align, bss_align)
        } else {
            (1, 1)
        };

        // version 3
        let fix_size = if version >= 3 { reader.bu32()? } else { 0 };

        ensure!(
            version <= 3,
            ParseProblem::UnsupportedVersion(version as usize, std::panic::Location::current())
        );
        ensure!(
            section_count > 1,
            ParseProblem::InvalidRange("no sections", std::panic::Location::current())
        );
        ensure!(
            section_count < 32,
            ParseProblem::InvalidRange(
                "section count limit exceeded",
                std::panic::Location::current()
            )
        );
        ensure!(
            section_offset >= 0x40,
            ParseProblem::InvalidRange("section offset < 0x40", std::panic::Location::current())
        );

        let prolog = optional_symbol(prolog_section, prolog_offset);
        let epilog = optional_symbol(epilog_section, epilog_offset);
        let unresolved = optional_symbol(unresolved_section, unresolved_offset);

        let sections = parse_sections(&mut reader, base, section_offset, section_count)?;
        let import_tables = parse_imports(&mut reader, base, import_offset, import_size)?;

        Ok(Rel {
            module,
            version,
            name_offset,
            name_size,
            sections,
            import_tables,
            prolog,
            epilog,
            unresolved,
            alignment: align,
            bss_alignment: bss_align,
            fix_size,
            relocation_offset: Some(relocation_offset),
            import_offset: Some(import_offset),
            import_size: Some(import_size),
        })
    }

    /// Relocation iterator.
    pub fn relocations(&self) -> RelocationIterator {
        RelocationIterator {
            rel:     self,
            table:   0,
            section: None,
            index:   0,
            offset:  0,
        }
    }
}

fn optional_symbol(section: u8, offset: u32) -> Option<Symbol> {
    if section != 0 {
        Some(Symbol {
            section: section as u32,
            offset,
        })
    } else {
        None
    }
}

fn parse_sections<D: Parser + Seeker>(
    reader: &mut D,
    base: u64,
    section_offset: u32,
    section_count: u32,
) -> Result<Vec<Section>> {
    let mut sections = Vec::<Section>::with_capacity(section_count as usize);
    for i in 0..section_count {
        let section_offset = base + section_offset as u64 + i as u64 * 8;
        reader.goto(section_offset)?;
        let offset_flags = reader.bu32()?; // ooooooff (o = offset, f = flags)
        let offset = offset_flags & !0x3_u32;
        let flags = offset_flags & 0x3_u32;
        let size = reader.bu32()?;

        let data = if offset > 0 {
            ensure!(
                size <= 0x2000000,
                ParseProblem::InvalidRange("section too large", std::panic::Location::current())
            );
            reader.goto(base + offset as u64)?;
            reader.read_as_vec(size as usize)?
        } else {
            Vec::new()
        };

        sections.push(Section {
            offset,
            size,
            executable: flags & 1 != 0,
            unknown: flags & 2 != 0,
            data,
        });
    }

    Ok(sections)
}

fn parse_imports<D: Parser + Seeker>(
    reader: &mut D,
    base: u64,
    import_offset: u32,
    import_size: u32,
) -> Result<Vec<ImportTable>> {
    let mut import_tables = Vec::<ImportTable>::new();
    let import_table_count = import_size / 8;
    for i in 0..import_table_count {
        reader.goto(base + (import_offset + i * 8) as u64)?;
        let module = reader.bu32()?;
        let offset = reader.bu32()?;

        let mut imports = Vec::new();
        reader.goto(base + offset as u64)?;
        loop {
            let offset = reader.bu16()?;
            let kind = reader.u8()?;
            let section = reader.u8()?;
            let addend = reader.bu32()?;

            let kind = match kind {
                0 => ImportKind::None,
                1 => ImportKind::Addr32,
                2 => ImportKind::Addr24,
                3 => ImportKind::Addr16,
                4 => ImportKind::Addr16Lo,
                5 => ImportKind::Addr16Hi,
                6 => ImportKind::Addr16Ha,
                7 => ImportKind::Addr14,
                10 => ImportKind::Rel24,
                11 => ImportKind::Rel14,
                201 => ImportKind::DolphinNop,
                202 => ImportKind::DolphinSection,
                203 => ImportKind::DolphinEnd,
                204 => ImportKind::DolphinMRKREF,
                _ => {
                    return Err(ParseProblem::InvalidData(
                        "unknown import kind",
                        std::panic::Location::current(),
                    )
                    .into())
                },
            };

            imports.push(Import {
                kind,
                section,
                offset,
                addend,
            });

            if kind == ImportKind::DolphinEnd {
                break;
            }
        }

        import_tables.push(ImportTable {
            module,
            offset,
            imports,
        });
    }

    Ok(import_tables)
}

/// An iterator over the relocations in [`Rel`].
pub struct RelocationIterator<'rel> {
    rel:     &'rel Rel,
    table:   usize,
    section: Option<usize>,
    index:   usize,
    offset:  u32,
}

impl<'rel> Iterator for RelocationIterator<'rel> {
    type Item = Relocation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.table >= self.rel.import_tables.len() {
            return None;
        }

        let table = &self.rel.import_tables[self.table];
        if self.index >= table.imports.len() {
            self.index = 0;
            self.table += 1;
            return self.next();
        }

        let import = &table.imports[self.index];
        self.index += 1;

        match import.kind {
            ImportKind::None | ImportKind::DolphinEnd => self.next(),
            ImportKind::DolphinMRKREF => {
                // unknown relocation type. The PowerPC Application Binary
                // Interface includes the relocation type "R_PPC_EMB_MRKREF"
                // which could be related. My understanding is that it's used to
                // mark a section as active even if it's not referenced.
                self.next()
            },
            ImportKind::DolphinNop => {
                // DolphinNop is used to support long offset values,
                // otherwise the offset field is limited to 16 bits. Any offets
                // that requires more bits are divided into one or more NOPs +
                // original relocation type.
                self.offset += import.offset as u32;
                self.next()
            },
            ImportKind::DolphinSection => {
                self.section = Some(import.section as usize);
                self.offset = 0;
                self.next()
            },
            kind => {
                if let Some(section) = self.section {
                    let relocation = Relocation {
                        kind,
                        module: table.module,
                        reference: SectionOffset {
                            section: import.section as u32,
                            offset:  import.addend,
                        },
                        target: SectionOffset {
                            section: section as u32,
                            offset:  self.offset + import.offset as u32,
                        },
                    };

                    self.offset = relocation.target.offset;
                    Some(relocation)
                } else {
                    // TODO: return error?
                    None
                }
            },
        }
    }
}
