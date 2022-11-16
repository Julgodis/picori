//! Revolution ARChive
//!
//! This file format is used in various GameCube and Wii games.
//!
//! It has been tested against Zelda: Twilight Princess for now.

use std::panic::Location;

use crate::error::ParseProblem;
use crate::helper::{ensure, Parser, ProblemLocation, Seeker};
use crate::{Ascii, Result, Yaz0Reader};

#[derive(Debug)]
struct Header {
    magic: u32,
    _size: u32,
    data_header_offset: u32,
    file_data_section_offset: u32,
    _file_data_section_length: u32,
    _mram_files_size: u32,
    _aram_files_size: u32,
    _dvd_files_size: u32,
}

impl Header {
    fn from_binary<D: Parser>(input: &mut D) -> Result<Self> {
        Ok(Self {
            magic: input.bu32()?,
            _size: input.bu32()?,
            data_header_offset: input.bu32()?,
            file_data_section_offset: input.bu32()?,
            _file_data_section_length: input.bu32()?,
            _mram_files_size: input.bu32()?,
            _aram_files_size: input.bu32()?,
            _dvd_files_size: input.bu32()?,
        })
    }

    fn is_valid(&self) -> bool {
        self.magic == 0x52415243
    }
}

#[derive(Debug)]
struct DataHeader {
    nb_dirs: u32,
    dirs_offset: u32,
    nb_files: u32,
    files_offset: u32,
    _strings_size: u32,
    strings_offset: u32,
    _next_file_index: u16,
    // TODO: Should be bool instead.
    _keep_file_id_sync: u8,
}

impl DataHeader {
    fn from_binary<D: Parser>(input: &mut D) -> Result<Self> {
        let header = Self {
            nb_dirs: input.bu32()?,
            dirs_offset: input.bu32()?,
            nb_files: input.bu32()?,
            files_offset: input.bu32()?,
            _strings_size: input.bu32()?,
            strings_offset: input.bu32()?,
            _next_file_index: input.bu16()?,
            _keep_file_id_sync: input.u8()?,
        };
        let padding = input.u8_array::<5>()?;
        ensure!(
            padding == [0, 0, 0, 0, 0],
            ParseProblem::InvalidData("invalid padding", Location::current())
        );
        Ok(header)
    }
}

#[derive(Debug)]
struct DirectoryNode {
    _name: String,
    name_offset: u32,
    name_hash: u16,
    nb_files: u16,
    files_offset: u32,
}

impl DirectoryNode {
    fn from_binary<D: Parser>(input: &mut D) -> Result<Self> {
        Ok(Self {
            _name: input.str::<4, Ascii>()?,
            name_offset: input.bu32()?,
            name_hash: input.bu16()?,
            nb_files: input.bu16()?,
            files_offset: input.bu32()?,
        })
    }
}

bitflags::bitflags! {
    struct FileAttribute: u8 {
        const FILE = 0x01;
        const DIRECTORY = 0x02;
        const COMPRESSED = 0x04;
        const PRELOAD_TO_MRAM = 0x10;
        const PRELOAD_TO_ARAM = 0x20;
        const LOAD_FROM_DVD = 0x40;
        const YAZ0_COMPRESSED = 0x80;
    }
}

#[derive(Debug)]
struct FileNode {
    index: u16,
    name_hash: u16,
    attrs: FileAttribute,
    _padding: u8,
    name_offset: u16,
    offset: u32,
    size: u32,
}

impl FileNode {
    fn from_binary<D: Parser>(input: &mut D) -> Result<Self> {
        let file = Self {
            index: input.bu16()?,
            name_hash: input.bu16()?,
            attrs: FileAttribute::from_bits(input.u8()?).unwrap(),
            _padding: input.u8()?,
            name_offset: input.bu16()?,
            offset: input.bu32()?,
            size: input.bu32()?,
        };
        let padding = input.u8_array::<4>()?;
        ensure!(
            padding == [0, 0, 0, 0],
            ParseProblem::InvalidData("invalid padding", Location::current())
        );
        Ok(file)
    }
}

/// Revolution ARChive, an archive format used in multiple GameCube and Wii games.
pub struct Rarc<D: Parser + Seeker> {
    reader: D,
    header: Header,
    data_header: DataHeader,
    directories: Vec<DirectoryNode>,
    files: Vec<FileNode>,
}

impl<D: Parser + Seeker> Rarc<D> {
    /// Parse a RARC file.
    pub fn new(mut reader: D) -> Result<Self> {
        let header = Header::from_binary(&mut reader)?;
        ensure!(
            header.is_valid(),
            ParseProblem::InvalidMagic("invalid magic", Location::current())
        );
        let base = header.data_header_offset as u64;
        reader.goto(base)?;
        let data_header = DataHeader::from_binary(&mut reader)?;
        let nb_dirs = data_header.nb_dirs;
        let nb_files = data_header.nb_files;

        // Parse all directory nodes.
        reader.goto(base + data_header.dirs_offset as u64)?;
        let mut directories = Vec::new();
        for _ in 0..data_header.nb_dirs {
            let dir = DirectoryNode::from_binary(&mut reader)?;
            directories.push(dir);
        }

        // Parse all file nodes.
        reader.goto(base + data_header.files_offset as u64)?;
        let mut files = Vec::new();
        for _ in 0..data_header.nb_files {
            let file = FileNode::from_binary(&mut reader)?;
            files.push(file);
        }

        let mut rarc = Self {
            reader,
            header,
            data_header,
            directories,
            files,
        };

        // Verify name integrity in both dirs and files.
        fn hash_name(name: &str) -> u16 {
            name.as_bytes()
                .iter()
                .fold(0, |hash, &c| hash * 3 + c as u16)
        }
        for i in 0..nb_dirs as usize {
            let name_offset = rarc.directories[i].name_offset as u64;
            let name = rarc.get_string(name_offset)?;
            let hash = hash_name(&name);
            let dir = &rarc.directories[i];
            assert_eq!(dir.name_hash, hash);
        }
        for i in 0..nb_files as usize {
            let name = rarc.get_file_name(i)?;
            let hash = hash_name(&name);
            let file = &rarc.files[i];
            assert_eq!(file.name_hash, hash);
        }

        Ok(rarc)
    }

    /// Extract the data of a given file, uncompressed if compressed.
    fn is_dir(&mut self, id: usize) -> bool {
        let file = &self.files[id];
        file.index == 0xffff
    }

    /// Extract the data of a given file, uncompressed if compressed.
    fn get_string(&mut self, offset: u64) -> Result<String> {
        let strings_offset = self.data_header.strings_offset as u64;
        self.reader.goto(0x20 + strings_offset + offset)?;
        Ok(self.reader.str::<256, Ascii>()?)
    }

    /// Extract the data of a given file, uncompressed if compressed.
    fn get_file_name(&mut self, id: usize) -> Result<String> {
        let file = &self.files[id];
        let filename_offset = file.name_offset as u64;
        self.get_string(filename_offset)
    }

    /// Extract the data of a given file, uncompressed if compressed.
    fn get_data(&mut self, id: usize) -> Result<Vec<u8>> {
        let file = &self.files[id];
        self.reader.goto(0x20 + self.header.file_data_section_offset as u64 + file.offset as u64)?;
        let data = self.reader.read_as_vec(file.size as usize)?;
        ensure!(
            file.attrs.contains(FileAttribute::FILE),
            ParseProblem::InvalidData("Not a file", Location::current())
        );
        Ok(if file.attrs.contains(FileAttribute::COMPRESSED | FileAttribute::YAZ0_COMPRESSED) {
            let mut cursor = std::io::Cursor::new(data);
            let mut yaz0 = Yaz0Reader::new(&mut cursor)?;
            let data = yaz0.decompress()?;
            data
        } else {
            data
        })
    }

    /// Return an iterator over the entries within a directory.
    pub fn read_dir(&mut self, path: &std::path::Path) -> Result<ReadDir> {
        let mut directory_id = 0;
        for component in path.components() {
            match component {
                std::path::Component::Normal(normal) => {
                    let directory = &self.directories[directory_id];
                    let begin = directory.files_offset as usize;
                    let end = begin + directory.nb_files as usize;
                    let mut found = false;
                    for file_id in begin..end {
                        let filename = self.get_file_name(file_id)?;
                        if filename == normal.to_str().unwrap() {
                            let file = &self.files[file_id];
                            directory_id = file.offset as usize;
                            found = true;
                        }
                    }
                    ensure!(
                        found,
                        ParseProblem::InvalidRange("File not found", Location::current())
                    );
                }
                _ => todo!()
            }
        }
        let directory = &self.directories[directory_id];
        Ok(ReadDir {
            cur: directory.files_offset as usize,
            last: directory.files_offset as usize + directory.nb_files as usize,
        })
    }
}

/// Iterator for a given RARC directory.
pub struct ReadDir {
    cur: usize,
    last: usize,
}

impl Iterator for ReadDir {
    type Item = DirEntry;

    fn next(&mut self) -> Option<DirEntry> {
        if self.cur == self.last {
            None
        } else {
            let entry = DirEntry {
                id: self.cur,
            };
            self.cur += 1;
            Some(entry)
        }
    }
}

/// Entry for a file inside of a RARC directory.
pub struct DirEntry {
    id: usize,
}

impl DirEntry {
    /// Return whether the entry is a directory or not.
    pub fn is_dir<D: Parser + Seeker>(&self, rarc: &mut Rarc<D>) -> bool {
        rarc.is_dir(self.id)
    }

    /// Return the name of this entry.
    pub fn name<D: Parser + Seeker>(&self, rarc: &mut Rarc<D>) -> Result<String> {
        rarc.get_file_name(self.id)
    }

    /// Return the data of the entry, if this is a file.
    pub fn data<D: Parser + Seeker>(&self, rarc: &mut Rarc<D>) -> Result<Vec<u8>> {
        rarc.get_data(self.id)
    }
}
