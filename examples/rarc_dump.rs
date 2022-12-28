use std::path::PathBuf;

use clap::Parser as ClapParser;
use picori::{rarc, RarcReader};

extern crate picori;

/// Simple program to greet a person
#[derive(ClapParser, Debug)]
#[command(
    name = "rarc_dump",
    bin_name = "rarc_dump",
    author="Julgodis <self@julgodis.xyz>", 
    version=env!("CARGO_PKG_VERSION"), 
    about="Example program to dump .arc (.rarc) files using picori", 
    long_about = None)]
struct Args {
    /// Path to the file to dump
    #[arg()]
    path: PathBuf,
    /// Dump directories
    #[arg(short, long)]
    directory: bool,
    /// Dump files
    #[arg(short, long)]
    file: bool,
    /// Dump tree
    #[arg(short, long)]
    tree: bool,
    /// Extract
    #[arg(short, long)]
    extract_path: Option<PathBuf>,

    /// Dump all
    #[arg(short, long)]
    all: bool,
    /// Column width
    #[arg(short, long, default_value = "32")]
    width: usize,
}

fn hex4(value: u16) -> String {
    format!("\x1b[36m{:#06x}\x1b[0m", value)
}

fn hex8(value: u32) -> String {
    format!("\x1b[36m{:#010x}\x1b[0m", value)
}

fn hex16(value: u64) -> String {
    format!("\x1b[36m{:#018x}\x1b[0m", value)
}

fn output_data(data: &Vec<u8>, width: usize) {
    for (j, line) in data.chunks(width).enumerate() {
        print!("{:06x}: ", j * 32);
        for byte in line {
            print!("{:02x} ", byte);
        }
        println!();
    }
}

fn output_directory<T>(reader: &RarcReader<T>)
where
    T: picori::Parser + picori::Seeker,
{
    println!("nodes:");
    for dir in reader.nodes() {
        match dir {
            rarc::Node::File { name, offset, size } => {
                println!("  file: {} ({})", name, hex4(name.hash));
                println!("    offset: {}", hex16(offset));
                println!("    size:   {}", hex8(size));
            },
            rarc::Node::DirectoryBegin { name } => {
                println!("  folder: {} ({})", name, hex4(name.hash));
            },
            _ => {},
        }
    }
}

fn output_file<T>(reader: &mut RarcReader<T>, width: usize) -> Result<(), picori::Error>
where
    T: picori::Parser + picori::Seeker,
{
    let files = reader
        .nodes()
        .filter_map(|x| match x {
            rarc::Node::File { name, offset, size } => Some((name, offset, size)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (name, offset, size) in files {
        println!("file: {} ({})", name, hex4(name.hash));
        println!("  offset: {}", hex16(offset));
        println!("  size:   {}", hex8(size));

        println!("  data:");
        let data = reader.file_data(offset, size)?;
        output_data(&data, width);
    }

    Ok(())
}

fn output_tree<T>(reader: &RarcReader<T>)
where
    T: picori::Parser + picori::Seeker,
{
    let mut indent = String::new();
    for node in reader.nodes() {
        let prefix = if false { "└" } else { "├" };

        match node {
            rarc::Node::File { name, .. } => {
                println!("{indent}{prefix} {}", name);
            },
            rarc::Node::DirectoryBegin { name } => {
                println!("{indent}{prefix} {}", name);
                indent.push_str("│");
            },
            rarc::Node::DirectoryEnd { .. } => {
                println!("{indent}┴");
                indent.pop();
            },
            rarc::Node::CurrentDirectory { .. } => {},
            rarc::Node::ParentDirectory { .. } => {},
        }
    }
}

fn main() -> Result<(), picori::Error> {
    let args = Args::parse();

    let mut dump_directory = args.directory;
    let mut dump_file = args.file;
    let mut dump_tree = args.tree;

    let out_path = args
        .extract_path
        .and_then(|x| std::fs::canonicalize(x).ok());

    let width = match args.width {
        0 => 1,
        _ => args.width,
    };

    if args.all {
        dump_directory = true;
        dump_file = true;
        dump_tree = true;
    }

    if !dump_directory && !dump_file && !dump_tree && out_path.is_none() {
        println!("nothing to dump :(");
        return Ok(());
    }

    let file = std::fs::File::open(&args.path)?;
    let file = std::io::BufReader::new(file);
    let file = picori::Yaz0Reader::new(file)?;
    let mut reader = RarcReader::new(file)?;

    if dump_directory {
        output_directory(&reader);
    }

    if dump_file {
        output_file(&mut reader, width)?;
    }

    if dump_tree {
        output_tree(&mut reader);
    }

    if let Some(out_path) = out_path {
        let mut path = out_path;
        let mut files = Vec::new();
        for node in reader.nodes() {
            match node {
                rarc::Node::File { name, offset, size } => {
                    let path = path.join(name.to_string());
                    files.push((path, offset, size));
                },
                rarc::Node::DirectoryBegin { name } => {
                    path.push(name.to_string());
                },
                rarc::Node::DirectoryEnd { .. } => {
                    path.pop();
                },
                _ => {},
            }
        }

        for (path, offset, length) in files {
            println!("extracting {}...", path.display());
            std::fs::create_dir_all(path.parent().expect("no parent"))?;

            let data = reader.file_data(offset, length)?;
            std::fs::write(&path, data)?;
        }
    }

    Ok(())
}
