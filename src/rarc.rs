use crate::error::ParseProblem;
use crate::helper::ProblemLocation;
use crate::helper::{ensure, Parser, Seeker};
use crate::{Ascii, Result};
use std::collections::HashMap;
use std::fmt::Display;
use std::panic::Location;

#[derive(Debug, Clone)]
pub struct NamedHash {
    pub name: String,
    pub hash: u16,
}

impl Display for NamedHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::hash::Hash for NamedHash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for NamedHash {
    fn eq(&self, other: &Self) -> bool {
        if self.hash == other.hash {
            self.name == other.name
        } else {
            false
        }
    }
}

impl Eq for NamedHash {}

#[derive(Debug, Clone)]
enum RarcDirectory {
    File {
        /// Name of the file.
        name: NamedHash,
        /// Offset of the file in the RARC file. This offset is relative to the start of the RARC file.
        offset: u64,
        /// Size of the file.
        size: u32,
    },
    Folder {
        /// Name of the folder.
        name: NamedHash,
    },
    CurrentFolder,
    ParentFolder,
}

#[derive(Debug, Clone)]
struct RarcNode {
    /// Index of first directory.
    pub index: u32,
    /// Number of directories.
    pub count: u32,
}

pub struct RarcReader<Reader> {
    reader: Reader,
    directories: Vec<RarcDirectory>,
    nodes: HashMap<NamedHash, RarcNode>,
    root_node: NamedHash,
}

impl<Reader: Parser + Seeker> RarcReader<Reader> {
    /// Creates a new RARC reader.
    pub fn new(mut reader: Reader) -> Result<Self> {
        let base = reader.position()?;

        let magic = reader.u32()?;
        let _file_length = reader.bu32()?;
        let header_length = reader.bu32()?;
        let file_offset = reader.bu32()?;
        let _file_length = reader.bu32()?;
        let _ = reader.bu32()?;
        let _ = reader.bu32()?;
        let _ = reader.bu32()?;
        let node_count = reader.bu32()?;
        let node_offset = reader.bu32()?;
        let directory_count = reader.bu32()?;
        let directory_offset = reader.bu32()?;
        let string_table_length = reader.bu32()?;
        let string_table_offset = reader.bu32()?;
        let _file_count = reader.bu16()?;
        let _ = reader.bu16()?;
        let _ = reader.bu32()?;

        ensure!(
            magic == 0x43524152,
            ParseProblem::InvalidMagic("invalid RARC magic", Location::current())
        );

        ensure!(
            node_count < 0x10000,
            ParseProblem::InvalidHeader("invalid node count", Location::current())
        );

        ensure!(
            directory_count < 0x10000,
            ParseProblem::InvalidHeader("invalid directory count", Location::current())
        );

        let base = base + header_length as u64;
        let directory_base = base + directory_offset as u64;
        let data_base = base + file_offset as u64;
        let mut directories = Vec::with_capacity(directory_count as usize);
        for i in 0..directory_count {
            reader.goto(directory_base + 20 * i as u64)?;
            let index = reader.bu16()?;
            let name_hash = reader.bu16()?;
            let _ = reader.bu16()?; // 0x200 for folders, 0x1100 for files
            let name_offset = reader.bu16()?;
            let data_offset = reader.bu32()?;
            let data_length = reader.bu32()?;
            let _ = reader.bu32()?;

            let name = {
                let offset = string_table_offset as u64;
                let offset = offset + name_offset as u64;
                ensure!(
                    (name_offset as u32) < string_table_length,
                    ParseProblem::InvalidData(
                        "invalid string table offset",
                        std::panic::Location::current()
                    )
                );
                reader.goto(base + offset)?;
                reader.str::<Ascii>()
            }?;

            if index == 0xFFFF {
                if name == "." {
                    directories.push(RarcDirectory::CurrentFolder);
                } else if name == ".." {
                    directories.push(RarcDirectory::ParentFolder);
                } else {
                    directories.push(RarcDirectory::Folder {
                        name: NamedHash {
                            name,
                            hash: name_hash,
                        },
                    });
                }
            } else {
                directories.push(RarcDirectory::File {
                    name: NamedHash {
                        name,
                        hash: name_hash,
                    },
                    offset: data_base + data_offset as u64,
                    size: data_length,
                });
            }
        }

        let node_base = base + node_offset as u64;
        let mut root_node: Option<NamedHash> = None;
        let mut nodes = HashMap::with_capacity(node_count as usize);
        for i in 0..node_count {
            reader.goto(node_base + 16 * i as u64)?;
            let _identifier = reader.bu32()?;
            let name_offset = reader.bu32()?;
            let name_hash = reader.bu16()?;
            let count = reader.bu16()? as u32;
            let index = reader.bu32()?;

            ensure!(
                index < directory_count,
                ParseProblem::InvalidData(
                    "first directory index out of bounds",
                    std::panic::Location::current()
                )
            );

            let last_index = index.checked_add(count);
            ensure!(
                last_index.is_some() && last_index.unwrap() <= directory_count,
                ParseProblem::InvalidData(
                    "last directory index out of bounds",
                    std::panic::Location::current()
                )
            );

            let name = {
                let offset = string_table_offset as u64;
                let offset = offset + name_offset as u64;
                ensure!(
                    (name_offset as u32) < string_table_length,
                    ParseProblem::InvalidData(
                        "invalid string table offset",
                        std::panic::Location::current()
                    )
                );
                reader.goto(base + offset)?;
                reader.str::<Ascii>()
            }?;

            // FIXME: this assumes that the root node is the first node in the list
            if root_node.is_none() {
                root_node = Some(NamedHash {
                    name: name.clone(),
                    hash: name_hash,
                });
            }

            let name = NamedHash {
                name,
                hash: name_hash,
            };
            nodes.insert(name.clone(), RarcNode { index, count });
        }

        if let Some(root_node) = root_node {
            Ok(Self {
                reader,
                directories,
                nodes,
                root_node,
            })
        } else {
            Err(ParseProblem::InvalidData("no root node", std::panic::Location::current()).into())
        }
    }

    /// Get the data for a file.
    pub fn file_data(&mut self, offset: u64, size: u32) -> Result<Vec<u8>> {
        self.reader.goto(offset)?;
        let mut result = vec![0; size as usize];
        self.reader.read_exact(result.as_mut_slice())?;
        Ok(result)
    }

    /// Get a iterator over the nodes in the RARC file.
    pub fn nodes(&self) -> Nodes<'_, Reader> {
        let root_node = self.root_node.clone();
        Nodes {
            parent: self,
            stack: vec![NodeState::Begin(root_node)],
        }
    }
}

/// A node in an RARC file.
pub enum Node {
    /// A directory that has been entered.
    DirectoryBegin { name: NamedHash },
    /// A directory that has been exited.
    DirectoryEnd { name: NamedHash },
    /// A file in the current directory.
    File {
        name: NamedHash,
        offset: u64,
        size: u32,
    },
    /// The current directory. This is equivalent to ".".
    CurrentDirectory,
    /// The parent directory. This is equivalent to "..".
    ParentDirectory,
}

enum NodeState {
    Begin(NamedHash),
    End(NamedHash),
    File(NamedHash, u32),
}

/// An iterator over the nodes in an RARC file.
pub struct Nodes<'parent, Reader> {
    parent: &'parent RarcReader<Reader>,
    stack: Vec<NodeState>,
}

impl<'parent, T: Parser + Seeker> Iterator for Nodes<'parent, T> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(state) = self.stack.pop() else {
            return None;
        };

        match state {
            NodeState::Begin(name) => {
                self.stack.push(NodeState::File(name.clone(), 0));
                Some(Node::DirectoryBegin { name })
            },
            NodeState::End(name) => Some(Node::DirectoryEnd { name }),
            NodeState::File(name, index) => {
                if let Some(node) = self.parent.nodes.get(&name) {
                    if index + 1 >= node.count {
                        self.stack.push(NodeState::End(name.clone()));
                    } else {
                        self.stack.push(NodeState::File(name.clone(), index + 1));
                    }
                    let directory = &self.parent.directories[(node.index + index) as usize];
                    match directory {
                        RarcDirectory::CurrentFolder => Some(Node::CurrentDirectory),
                        RarcDirectory::ParentFolder => Some(Node::ParentDirectory),
                        RarcDirectory::Folder { name } => {
                            self.stack.push(NodeState::Begin(name.clone()));
                            self.next()
                        },
                        RarcDirectory::File { name, offset, size } => Some(Node::File {
                            name: name.clone(),
                            offset: *offset,
                            size: *size,
                        }),
                    }
                } else {
                    None
                }
            },
        }
    }
}
