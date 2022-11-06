//! Parse and build `.dol` files.
//!
//! ## Parse
//!
//! To parse a `.dol` file, use [`from_bytes`]. The section data is referenced
//! from the bytes passed to [`from_bytes`], thus ['Dol'] struct is only valid
//! for as long as those bytes are available.
//!
//! First we need to have access to the `.dol` file bytes. E.g., we can read
//! the file into a `Vec<u8>` using [`std::fs::read`]. Next step is to parse
//! the bytes into a [`Dol`] struct using [`from_bytes`]. If no error occurs,
//! we can access the sections using the [`sections`] field or the entrypoint
//! using the [`entrypoint`] method.
//! ```no_run
//! use anyhow::Result;
//! fn main() -> Result<()> {
//!     let data = std::fs::read("../../assets/gzle01.dol")?;
//!     let dol = picori::format::dol::from_bytes(&data)?;
//!     println!("Entrypoint: 0x{:08x}", dol.entrypoint());
//!     Ok(())
//! }
//! ```
//!
//! ## Build
//!
//! TODO: Write this section.

use anyhow::{ensure, Result};
use itertools::{chain, izip};

use crate::error::DolError;
use crate::helper::{align_next, checked_add, read_bu32, read_bu32_array, TakeLastN};

/// The `.dol` header without any modifications. This is the raw data that is
/// read from the file. The data has been endian-flipped to be in the native
/// endian format.
#[derive(Debug)]
pub struct Header {
    pub text_offset:  [u32; 7],  // 0x00
    pub data_offset:  [u32; 11], // 0x1C
    pub text_address: [u32; 7],  // 0x48
    pub data_address: [u32; 11], // 0x64
    pub text_size:    [u32; 7],  // 0x90
    pub data_size:    [u32; 11], // 0xAC
    pub bss_address:  u32,       // 0xD8
    pub bss_size:     u32,       // 0xDC
    pub entrypoint:   u32,       // 0xE0
}

#[derive(Debug)]
pub enum SectionKind {
    Text,
    Data,
    Bss,
}

///
#[derive(Debug)]
pub struct Section<'a> {
    /// The kind of section this is (text, data, or bss).
    pub kind: SectionKind,

    /// The section name (e.g. `.text`, `.data`, `.rodata`, etc.), this was
    /// guessed from the type of section and order in which they appear in
    /// the `.dol`. This is not guaranteed to be correct, as the `.dol`
    /// format does not specify the name of the section.
    pub name: &'static str,

    /// The section address that the data is loaded to in memory on startup.
    pub address: u32,

    /// The section size in bytes.
    pub size: u32,

    /// The section size in bytes, rounded up to the nearest multiple of 32.
    pub aligned_size: u32,

    /// The section data.
    pub data: &'a [u8],
}

/// RomCopyInfo represents one entry in the `__rom_copy_info` symbol generated
/// by the linker at the end of the `.init` section. It has information
/// otherwise lost in the process of converting `.elf` to `.dol`, such as, the
/// original unaligned section size. At startup the `__rom_copy_info` is used to
/// copy each entry from the ROM to the RAM.
#[derive(Debug)]
pub struct RomCopyInfo {
    /// Read Only Memory (ROM) address of the section.
    pub rom_address: u32,

    /// Random Access Memory (RAM) address of the section.
    pub ram_address: u32,

    /// The size of the section in bytes.
    pub size: u32,
}

/// BssInitInfo represents one entry in the `__bss_init_info` symbol generated
/// by the linker at the end of the `.init` section. It has information
/// otherwise lost in the process of converting `.elf` to `.dol`, such as, the
/// original unaligned section size and how many `.bss` (`.sbss`, `.bss2`, etc.)
/// sections exists. The final `.dol` file will have a single `.bss` section
/// with the size of the sum of all the `.bss` sections. At startup the
/// `__bss_init_info` is used to zero out the `.bss` section in RAM.
#[derive(Debug)]
pub struct BssInitInfo {
    /// Random Access Memory (RAM) address of the section.
    pub ram_address: u32,

    /// The size of the section in bytes.
    pub size: u32,
}

pub struct Dol<'a> {
    pub header:        Header,
    pub rom_copy_info: Option<Vec<RomCopyInfo>>,
    pub bss_init_info: Option<Vec<BssInitInfo>>,
    pub sections:      Vec<Section<'a>>,
}

impl RomCopyInfo {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() == 12, "invalid size for RomCopyInfo");
        let rom_address = read_bu32!(bytes, 0);
        let ram_address = read_bu32!(bytes, 4);
        let size = read_bu32!(bytes, 8);
        Ok(RomCopyInfo {
            rom_address,
            ram_address,
            size,
        })
    }
}

impl BssInitInfo {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() == 8, "invalid size for BssInitInfo");
        let ram_address = read_bu32!(bytes, 0);
        let size = read_bu32!(bytes, 4);
        Ok(BssInitInfo { ram_address, size })
    }
}

/// Search 0x200 bytes from the end of `data` (from the `.init` section)
/// until we find all `__rom_copy_info` entries.
fn rom_copy_info_search(data: &[u8], address: u32) -> Option<Vec<RomCopyInfo>> {
    Some(
        data.take_last_n(0x200)
            .windows(12)
            .map(|x| RomCopyInfo::from_bytes(x))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .skip_while(|x| x.rom_address != address || x.ram_address != address)
            .step_by(12)
            .take_while(|x| x.rom_address != 0)
            .collect(),
    )
}

/// Search 0x200 bytes from the end of `data` (from the `.init` section)
/// until we find all `__bss_init_info` entries.
fn bss_init_info_search(data: &[u8], address: u32) -> Option<Vec<BssInitInfo>> {
    Some(
        data.take_last_n(0x200)
            .windows(8)
            .map(|x| BssInitInfo::from_bytes(x))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .skip_while(|x| x.ram_address != address)
            .step_by(8)
            .take_while(|x| x.ram_address != 0)
            .collect(),
    )
}

fn section_name(kind: SectionKind, index: usize) -> &'static str {
    match kind {
        SectionKind::Text => match index {
            0 => ".init",
            1 => ".text",
            2 => ".text.2",
            3 => ".text.3",
            4 => ".text.4",
            5 => ".text.5",
            6 => ".text.6",
            _ => panic!("invalid text section index"),
        },
        SectionKind::Data => match index {
            0 => "extab_",
            1 => "extabindex_",
            2 => ".ctors",
            3 => ".dtors",
            4 => ".rodata",
            5 => ".data",
            6 => ".sdata",
            7 => ".sdata2",
            8 => ".data8",
            9 => ".data9",
            10 => ".data10",
            _ => panic!("invalid data section index"),
        },
        SectionKind::Bss => match index {
            0 => ".bss",
            1 => ".sbss",
            2 => ".sbss2",
            _ => panic!("invalid bss section index"),
        },
    }
}

pub fn from_bytes<'i>(bytes: &'i [u8]) -> Result<Dol<'i>> {
    ensure!(bytes.len() >= 0x100, DolError::InvalidHeaderSize {
        size: bytes.len(),
    });

    let header = Header {
        text_offset:  read_bu32_array!(bytes, 0x00, 7),
        data_offset:  read_bu32_array!(bytes, 0x1C, 11),
        text_address: read_bu32_array!(bytes, 0x48, 7),
        data_address: read_bu32_array!(bytes, 0x64, 11),
        text_size:    read_bu32_array!(bytes, 0x90, 7),
        data_size:    read_bu32_array!(bytes, 0xAC, 11),
        bss_address:  read_bu32!(bytes, 0xD8),
        bss_size:     read_bu32!(bytes, 0xDC),
        entrypoint:   read_bu32!(bytes, 0xE0),
    };

    for (i, (offset, size)) in header
        .text_offset
        .iter()
        .zip(header.text_size.iter())
        .enumerate()
    {
        if offset == &0 || size == &0 {
            continue;
        }

        let begin = *offset;
        let end = checked_add(begin, *size)?;
        ensure!(
            begin >= 0x100 && end <= bytes.len() as u32,
            DolError::TextSectionOutOfBounds { section: i }
        );
    }

    for (i, (offset, size)) in header
        .data_offset
        .iter()
        .zip(header.data_size.iter())
        .enumerate()
    {
        if offset == &0 || size == &0 {
            continue;
        }

        let begin = *offset;
        let end = checked_add(begin, *size)?;
        ensure!(
            begin >= 0x100 && end <= bytes.len() as u32,
            DolError::DataSectionOutOfBounds { section: i }
        );
    }

    let text_sections = izip!(
        header.text_offset.iter(),
        header.text_address.iter(),
        header.text_size.iter(),
    )
    .enumerate()
    .map(|(index, (offset, address, size))| Section {
        kind:         SectionKind::Text,
        name:         section_name(SectionKind::Text, index),
        address:      *address,
        size:         *size,
        aligned_size: *size,
        data:         &bytes[*offset as usize..(*offset + *size) as usize],
    });

    let data_sections = izip!(
        header.data_offset.iter(),
        header.data_address.iter(),
        header.data_size.iter(),
    )
    .enumerate()
    .map(|(index, (offset, address, size))| Section {
        kind:         SectionKind::Data,
        name:         section_name(SectionKind::Data, index),
        address:      *address,
        size:         *size,
        aligned_size: *size,
        data:         &bytes[*offset as usize..(*offset + *size) as usize],
    });

    let mut sections: Vec<Section> = chain!(text_sections, data_sections)
        .filter(|section| section.size != 0)
        .collect();

    let init = sections.iter().find(|x| x.name == ".init");
    let rom_copy_info = init.map_or(None, |init| rom_copy_info_search(init.data, init.address));

    let bss_init_info = init.map_or(None, |init| {
        bss_init_info_search(init.data, header.bss_address)
    });

    for section in sections.iter_mut() {
        section.size = rom_copy_info
            .as_ref()
            .and_then(|v| v.iter().find(|x| x.rom_address == section.address))
            .map_or(section.size, |x| x.size);
    }

    // If `__bss_init_info` is available we can use it to determine the size and
    // count of the `.bss` sections. Otherwise we assume that there is only one
    // `.bss` section and use the size from the header (which is probably
    // not correct).
    if let Some(bss_init_info) = &bss_init_info {
        let bss_sections = bss_init_info
            .iter()
            .enumerate()
            .map(|(index, entry)| Section {
                kind:         SectionKind::Bss,
                name:         section_name(SectionKind::Bss, index),
                address:      entry.ram_address,
                size:         entry.size,
                aligned_size: align_next(entry.size, 32),
                data:         &[],
            });
        sections.extend(bss_sections)
    } else {
        sections.push(Section {
            kind:         SectionKind::Bss,
            name:         section_name(SectionKind::Bss, 0),
            address:      header.bss_address,
            size:         header.bss_size,
            aligned_size: header.bss_size,
            data:         &[],
        });
    }

    Ok(Dol {
        header,
        rom_copy_info: rom_copy_info,
        bss_init_info: bss_init_info,
        sections: sections,
    })
}

pub fn to_bytes<'data>(_dol: &Dol<'data>) -> Result<Vec<u8>> {
    unimplemented!("picori::format::dol::to_bytes");
}

impl<'data> Dol<'data> {
    pub fn entrypoint(&self) -> u32 { self.header.entrypoint }

    pub fn section_by_name(&self, name: &str) -> Option<&Section<'data>> {
        self.sections.iter().find(|x| x.name == name)
    }

    pub fn section_by_address(&self, address: u32) -> Option<&Section<'data>> {
        self.sections
            .iter()
            .find(|x| x.address <= address && address < x.address + x.size)
    }

    pub fn from_bytes(bytes: &'data [u8]) -> Result<Dol<'data>> { from_bytes(bytes) }
}
