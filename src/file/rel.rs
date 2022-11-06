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
use std::io::SeekFrom;

use crate::error::DeserializeProblem::{self, *};
use crate::helper::{ensure, Deserializer, Seeker};
use crate::Result;

pub struct Rel {
    /// The module number. Must be unique per `.rel` file.
    pub module:        u32,
    /// The version number of the `.rel` file.
    pub version:       u32,
    /// The offset of the name of the module. The name is available in the
    /// `framework.str`.
    pub name_offset:   u32,
    /// The size of the name of the module.
    pub name_size:     u32,
    /// Sections.
    pub sections:      Vec<Section>,
    pub import_tables: Vec<ImportTable>,
    /// The prolog symbol.
    pub prolog:        Option<Symbol>,
    /// The epilog symbol.
    pub epilog:        Option<Symbol>,
    /// The unresolved symbol. This function is called when a symbol is not
    /// could not be resolved.
    pub unresolved:    Option<Symbol>,

    pub alignment:         u32,
    pub bss_alignment:     u32,
    pub fix_size:          u32,
    pub relocation_offset: Option<u32>,
    pub import_offset:     Option<u32>,
    pub import_size:       Option<u32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Section {
    pub offset:     u32,
    pub size:       u32,
    pub executable: bool,
    pub unknown:    bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImportKind {
    None,
    Addr32,
    Addr24,
    Addr16,
    Addr16Lo,
    Addr16Hi,
    Addr16Ha,
    Addr14,
    Rel24,
    Rel14,
    DolphinNop,
    DolphinSection,
    DolphinEnd,
    DolphinMRKREF,
}

pub struct Import {
    pub kind:    ImportKind,
    pub section: u8,
    pub offset:  u16,
    pub addend:  u32,
}

pub struct ImportTable {
    pub module:  u32,
    pub offset:  u32,
    pub imports: Vec<Import>,
}

/// A symbol reference.
pub struct Symbol {
    /// Section where the symbol is located.
    pub section: u32,
    /// Offset of the symbol in the section.
    pub offset:  u32,
}

pub enum RelocationKind {
    Addr32,
    Addr24,
    Addr16,
    Addr16Lo,
    Addr16Hi,
    Addr16Ha,
    Addr14,
    Rel24,
    Rel14,
}

/// Reference to bytes in a section.
pub struct Location {
    /// Section where the bytes are located.
    pub section: u32,
    /// Offset of the bytes in the section.
    pub offset:  u32,
}

pub struct Relocation {
    /// Relocation kind.
    pub kind:      RelocationKind,
    /// Module of the referenced symbol.
    pub module:    u32,
    /// Reference to the symbol.
    pub reference: Location,
    /// Where the relocation is located (the location of the bytes that should
    /// be modified).
    pub target:    Location,
}

pub struct RelocationIterator<'rel, D: Deserializer + Seeker> {
    rel:     &'rel RelReader<D>,
    table:   usize,
    section: Option<usize>,
    index:   usize,
    offset:  u32,
}

impl<'rel, D: Deserializer + Seeker> Iterator for RelocationIterator<'rel, D> {
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
            ImportKind::Addr32
            | ImportKind::Addr24
            | ImportKind::Addr16
            | ImportKind::Addr16Lo
            | ImportKind::Addr16Hi
            | ImportKind::Addr16Ha
            | ImportKind::Addr14
            | ImportKind::Rel24
            | ImportKind::Rel14 => {
                if let Some(section) = self.section {
                    let kind = match import.kind {
                        ImportKind::Addr32 => RelocationKind::Addr32,
                        ImportKind::Addr24 => RelocationKind::Addr24,
                        ImportKind::Addr16 => RelocationKind::Addr16,
                        ImportKind::Addr16Lo => RelocationKind::Addr16Lo,
                        ImportKind::Addr16Hi => RelocationKind::Addr16Hi,
                        ImportKind::Addr16Ha => RelocationKind::Addr16Ha,
                        ImportKind::Addr14 => RelocationKind::Addr14,
                        ImportKind::Rel24 => RelocationKind::Rel24,
                        ImportKind::Rel14 => RelocationKind::Rel14,
                        _ => unreachable!(),
                    };

                    let relocation = Relocation {
                        kind:      kind,
                        module:    table.module,
                        reference: Location {
                            section: import.section as u32,
                            offset:  import.addend,
                        },
                        target:    Location {
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
        }
    }
}

pub struct RelReader<D: Deserializer + Seeker> {
    reader: D,
    base:   u64,

    /// The module number. Must be unique per `.rel` file.
    pub module:        u32,
    /// The version number of the `.rel` file.
    pub version:       u32,
    /// The offset of the name of the module. The name is available in the
    /// `framework.str`.
    pub name_offset:   u32,
    /// The size of the name of the module.
    pub name_size:     u32,
    /// Sections.
    pub sections:      Vec<Section>,
    pub import_tables: Vec<ImportTable>,
    /// The prolog symbol.
    pub prolog:        Option<Symbol>,
    /// The epilog symbol.
    pub epilog:        Option<Symbol>,
    /// The unresolved symbol. This function is called when a symbol is not
    /// could not be resolved.
    pub unresolved:    Option<Symbol>,

    pub alignment:         u32,
    pub bss_alignment:     u32,
    pub fix_size:          u32,
    pub relocation_offset: Option<u32>,
    pub import_offset:     Option<u32>,
    pub import_size:       Option<u32>,
}

impl<D: Deserializer + Seeker> RelReader<D> {
    pub fn new(mut reader: D) -> Result<Self> {
        let base = reader.position()?;
        let module = reader.deserialize_bu32()?;
        let _next = reader.deserialize_bu32()?; // should be 0, used at runtime
        let _prev = reader.deserialize_bu32()?; // should be 0, used at runtime
        let section_count = reader.deserialize_bu32()?;
        let section_offset = reader.deserialize_bu32()?;
        let name_offset = reader.deserialize_bu32()?;
        let name_size = reader.deserialize_bu32()?;
        let version = reader.deserialize_bu32()?;
        let _bss_size = reader.deserialize_bu32()?;
        let relocation_offset = reader.deserialize_bu32()?;
        let import_offset = reader.deserialize_bu32()?;
        let import_size = reader.deserialize_bu32()?;
        let prolog_section = reader.deserialize_u8()?;
        let epilog_section = reader.deserialize_u8()?;
        let unresolved_section = reader.deserialize_u8()?;
        let _bss_section = reader.deserialize_u8()?; // should be 0, used at runtime
        let prolog_offset = reader.deserialize_bu32()?;
        let epilog_offset = reader.deserialize_bu32()?;
        let unresolved_offset = reader.deserialize_bu32()?;

        // version 2
        let (align, bss_align) = if version >= 2 {
            let align = reader.deserialize_bu32()?;
            let bss_align = reader.deserialize_bu32()?;
            (align, bss_align)
        } else {
            (1, 1)
        };

        // version 3
        let fix_size = if version >= 3 {
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

        let prolog = optional_symbol(prolog_section, prolog_offset);
        let epilog = optional_symbol(epilog_section, epilog_offset);
        let unresolved = optional_symbol(unresolved_section, unresolved_offset);

        let sections = parse_sections(&mut reader, base, section_offset, section_count)?;
        let import_tables = parse_imports(&mut reader, base, import_offset, import_size)?;

        Ok(RelReader {
            reader,
            base,
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

    pub fn section_at(&mut self, index: usize) -> Result<(Section, Vec<u8>)> {
        let section = *self
            .sections
            .get(index)
            .ok_or(InvalidData("invalid section index"))?;
        let mut data = vec![0; section.size as usize];
        self.section_at_into(index, data.as_mut_slice())?;
        Ok((section, data))
    }

    pub fn section_at_into(&mut self, index: usize, buf: &mut [u8]) -> Result<()> {
        let section = self
            .sections
            .get(index)
            .ok_or(InvalidData("invalid section index"))?;
        self.reader
            .seek(SeekFrom::Start(self.base + section.offset as u64))?;
        self.reader.read_into_buffer(buf)?;
        Ok(())
    }

    pub fn relocations<'this>(&'this self) -> RelocationIterator<'this, D> {
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
            offset:  offset,
        })
    } else {
        None
    }
}

fn parse_sections<D: Deserializer + Seeker>(
    reader: &mut D,
    base: u64,
    section_offset: u32,
    section_count: u32,
) -> Result<Vec<Section>> {
    let mut sections = Vec::<Section>::with_capacity(section_count as usize);
    reader.seek(std::io::SeekFrom::Start(base + section_offset as u64))?;
    for _ in 0..section_count {
        let offset_flags = reader.deserialize_bu32()?; // ooooooff (o = offset, f = flags)
        let offset = offset_flags & !0x3_u32;
        let flags = offset_flags & 0x3_u32;
        let size = reader.deserialize_bu32()?;

        sections.push(Section {
            offset,
            size,
            executable: flags & 1 != 0,
            unknown: flags & 2 != 0,
        });
    }

    Ok(sections)
}

fn parse_imports<D: Deserializer + Seeker>(
    reader: &mut D,
    base: u64,
    import_offset: u32,
    import_size: u32,
) -> Result<Vec<ImportTable>> {
    let mut import_tables = Vec::<ImportTable>::new();
    let import_table_count = import_size / 8;
    for i in 0..import_table_count {
        reader.seek(std::io::SeekFrom::Start(
            base + (import_offset + i * 8) as u64,
        ))?;
        let module = reader.deserialize_bu32()?;
        let offset = reader.deserialize_bu32()?;

        let mut imports = Vec::new();
        reader.seek(std::io::SeekFrom::Start(base + offset as u64))?;
        loop {
            let offset = reader.deserialize_bu16()?;
            let kind = reader.deserialize_u8()?;
            let section = reader.deserialize_u8()?;
            let addend = reader.deserialize_bu32()?;

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
                _ => return Err(DeserializeProblem::InvalidData("unknown import kind").into()),
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
