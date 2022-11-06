use std::io::SeekFrom;
use std::path::PathBuf;

use crate::error::ParseProblem;
use crate::helper::{ensure, ProblemLocation};
use crate::{Ascii, Deserializer, Result, Seeker};

/// Enum varient of a single [`Fst`] entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// Root directory.
    Root,
    /// File.
    File {
        /// Relative filename.
        name:   String,
        /// `internal`: Entry index.
        index:  u32,
        /// File offset from the beginning of the GCM file.
        offset: u32,
        /// File size.
        size:   u32,
    },
    /// Directory.
    Directory {
        /// Relative directory name.
        name:   String,
        /// `internal`: Parent index.
        parent: u32,
        /// `internal`: First child index.
        begin:  u32,
        /// `internal`: Last child index.
        end:    u32,
    },
}

enum RawEntry {
    File {
        name:   u32,
        offset: u32,
        size:   u32,
    },
    Directory {
        name:   u32,
        parent: u32,
        end:    u32,
    },
}

impl RawEntry {
    pub fn new<D: Deserializer + Seeker>(input: &mut D) -> Result<Self> {
        let flag_or_name_offset = input.deserialize_bu32()?;
        let data_offset_or_parent = input.deserialize_bu32()?;
        let data_length_or_end = input.deserialize_bu32()?;
        let flag = flag_or_name_offset >> 24;
        let name_offset = flag_or_name_offset & 0x00ffffff;

        if (flag & 1) == 0 {
            Ok(Self::File {
                name:   name_offset,
                offset: data_offset_or_parent,
                size:   data_length_or_end,
            })
        } else {
            Ok(Self::Directory {
                name:   name_offset,
                parent: data_offset_or_parent,
                end:    data_length_or_end,
            })
        }
    }
}

/// File String Table (`fst.bin`). The [`Fst`] contains information about the
/// file structure of the GameCube disc, i.e. the file names and their
/// locations.
pub struct Fst {
    entries: Vec<Entry>,
}

impl Fst {
    /// Parse GCM FST.
    ///
    /// To read the full string table, this function needs the size of the
    /// [`Fst`]. This is available in the [`Boot`] struct.
    pub fn deserialize<D: Deserializer + Seeker>(reader: &mut D, fst_size: usize) -> Result<Fst> {
        let base = reader.position()?;

        let _ = reader.deserialize_bu32()?;
        let _ = reader.deserialize_bu32()?;
        let root_count = reader.deserialize_bu32()?;
        let entry_count = root_count as usize;
        ensure!(
            entry_count <= 0x4000,
            ParseProblem::InvalidRange(
                "entry count limit (max 16384)",
                std::panic::Location::current()
            )
        );

        reader.seek(SeekFrom::Start(base))?;
        let temp_entries = (0..entry_count)
            .map(|_| RawEntry::new(reader))
            .collect::<Result<Vec<_>>>()?;

        let entry_size = 0x0C * entry_count;
        let name_table_size = fst_size - entry_size;
        let string_table = reader.read_as_vec(name_table_size)?;

        let mut entries = Vec::with_capacity(entry_count);
        for (i, entry) in temp_entries.iter().enumerate() {
            if i == 0 {
                entries.push(Entry::Root);
                continue;
            }

            let entry = match entry {
                RawEntry::File { name, offset, size } => Entry::File {
                    name:   Ascii::first(&string_table[*name as usize..])?,
                    index:  i as u32,
                    offset: *offset,
                    size:   *size,
                },
                RawEntry::Directory { name, parent, end } => Entry::Directory {
                    name:   Ascii::first(&string_table[*name as usize..])?,
                    parent: *parent,
                    begin:  (i + 1) as u32,
                    end:    *end,
                },
            };

            entries.push(entry);
        }

        Ok(Fst { entries })
    }

    /// Get an iterator over all [`FstEntry`]s.
    pub fn files(&self) -> FileIterator {
        FileIterator {
            fst: self,
            index: 0,
            last_index_of_directory: vec![],
            path: PathBuf::new(),
        }
    }
}

/// Iterator over all files in a [`Fst`].
pub struct FileIterator<'fst> {
    fst: &'fst Fst,
    index: usize,
    last_index_of_directory: Vec<usize>,
    path: PathBuf,
}

impl<'fst> Iterator for FileIterator<'fst> {
    type Item = (PathBuf, Entry);

    fn next(&mut self) -> Option<Self::Item> {
        while !self.last_index_of_directory.is_empty() {
            if self.last_index_of_directory.last() == Some(&self.index) {
                self.last_index_of_directory.pop();
                self.path.pop();
            } else {
                break;
            }
        }

        let entry = self.fst.entries.get(self.index)?;
        self.index += 1;

        match entry {
            Entry::File { name, .. } => {
                let path = self.path.join(name);
                Some((path, entry.clone()))
            },
            Entry::Directory {
                begin, end, name, ..
            } => {
                self.index = *begin as usize;
                self.last_index_of_directory.push(*end as usize);
                self.path.push(name);
                Some((self.path.clone(), entry.clone()))
            },
            Entry::Root => Some((self.path.clone(), entry.clone())),
        }
    }
}
