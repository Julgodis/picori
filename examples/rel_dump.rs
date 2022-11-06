use std::io::Cursor;
use std::path::PathBuf;

use clap::Parser;
use colored::{ColoredString, Colorize};
use picori::file::rel::{ImportKind, Rel, RelReader, RelocationKind};
use picori::{Deserializer, Seeker};

extern crate picori;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    name = "rel_dump",
    bin_name = "rel_dump",
    author="Julgodis <self@julgodis.xyz>", 
    version=env!("CARGO_PKG_VERSION"), 
    about="Example program to dump .rel files using picori", 
    long_about = None)]
struct Args {
    /// Path to the file to dump
    #[arg()]
    path:        PathBuf,
    /// Dump header
    #[arg(short = 't', long)]
    header:      bool,
    /// Dump sections
    #[arg(short, long)]
    sections:    bool,
    /// Dump imports
    #[arg(short, long)]
    imports:     bool,
    /// Dump relocations
    #[arg(short, long)]
    relocations: bool,
    /// Dump section data
    #[arg(short, long)]
    data:        bool,
    /// Dump all
    #[arg(short, long)]
    all:         bool,
    /// Column width
    #[arg(short, long, default_value = "32")]
    width:       usize,
}

fn hex4(value: u16) -> ColoredString { format!("{:#06x}", value).cyan() }
fn hex8(value: u32) -> ColoredString { format!("{:#010x}", value).cyan() }
fn num(value: u32) -> ColoredString { format!("{}", value).cyan() }

fn output_header<D: Deserializer + Seeker>(rel: &mut RelReader<D>) {
    println!("header:");
    println!("  module: {}", num(rel.module));
    println!("  version: {}", num(rel.version));
    println!("  name offset: {}", hex8(rel.name_offset));
    println!("  name size:   {}", hex8(rel.name_size));
    println!("  alignment:     {}", hex8(rel.alignment));
    println!("  bss alignment: {}", hex8(rel.bss_alignment));
    println!("  fix size:      {}", hex8(rel.fix_size));
    println!(
        "  relocation offset: {}",
        hex8(rel.relocation_offset.unwrap_or(0))
    );
    println!(
        "  import offset:     {}",
        hex8(rel.import_offset.unwrap_or(0))
    );
    println!(
        "  import size:       {}",
        hex8(rel.import_size.unwrap_or(0))
    );
    println!(
        "  prolog:     {}",
        if let Some(prolog) = &rel.prolog {
            format!("{} (section: {})", hex8(prolog.offset), num(prolog.section))
        } else {
            "".to_string()
        }
    );
    println!(
        "  epilog:     {}",
        if let Some(epilog) = &rel.epilog {
            format!("{} (section: {})", hex8(epilog.offset), num(epilog.section))
        } else {
            "".to_string()
        }
    );
    println!(
        "  unresolved: {}",
        if let Some(unresolved) = &rel.unresolved {
            format!(
                "{} (section: {})",
                hex8(unresolved.offset),
                num(unresolved.section)
            )
        } else {
            "".to_string()
        }
    );
}

fn output_sections<D: Deserializer + Seeker>(rel: &mut RelReader<D>) {
    println!("sections:");
    for (i, section) in rel.sections.iter().enumerate() {
        println!(
            "  #{:<2} offset: {}, size: {}{}{}{}{}",
            num(i as u32),
            hex8(section.offset),
            hex8(section.size),
            if section.executable {
                " [executable]".red()
            } else {
                ColoredString::default()
            },
            if section.unknown {
                " [unknown]".red()
            } else {
                ColoredString::default()
            },
            if i == 0 {
                " [null section]".green()
            } else {
                ColoredString::default()
            },
            if i != 0 && section.offset == 0 && section.size == 0 {
                " [unused]".green()
            } else {
                ColoredString::default()
            }
        );
    }
}

fn output_imports<D: Deserializer + Seeker>(rel: &mut RelReader<D>) {
    println!("import tables:");
    for (i, table) in rel.import_tables.iter().enumerate() {
        println!(
            "  #{:<2} module: {:>4}, size: {}",
            num(i as u32),
            num(table.module),
            hex8(table.offset)
        );
    }

    println!("import:");
    for table in rel.import_tables.iter() {
        println!("  [ module: {:>4} ]", num(table.module));
        for (j, import) in table.imports.iter().enumerate() {
            let kind = match import.kind {
                ImportKind::None => "None",
                ImportKind::Addr32 => "Addr32",
                ImportKind::Addr24 => "Addr24",
                ImportKind::Addr16 => "Addr16",
                ImportKind::Addr16Lo => "Addr16Lo",
                ImportKind::Addr16Hi => "Addr16Hi",
                ImportKind::Addr16Ha => "Addr16Ha",
                ImportKind::Addr14 => "Addr14",
                ImportKind::Rel24 => "Rel24",
                ImportKind::Rel14 => "Rel14",
                ImportKind::DolphinNop => "DolphinNop",
                ImportKind::DolphinSection => "DolphinSection",
                ImportKind::DolphinEnd => "DolphinEnd",
                ImportKind::DolphinMRKREF => "DolphinMRKREF",
                _ => "Unknown",
            };

            println!(
                "  #{:<4} {:<20} section: {:>2}, offset: {}, addend: {}",
                num(j as u32),
                kind,
                num(import.section as u32),
                hex4(import.offset),
                hex8(import.addend)
            );
        }
    }
}

fn output_relocations<D: Deserializer + Seeker>(rel: &mut RelReader<D>) {
    println!("relocation:");

    for (i, relocation) in rel.relocations().enumerate() {
        let kind = match relocation.kind {
            RelocationKind::Addr32 => "Addr32",
            RelocationKind::Addr24 => "Addr24",
            RelocationKind::Addr16 => "Addr16",
            RelocationKind::Addr16Lo => "Addr16Lo",
            RelocationKind::Addr16Hi => "Addr16Hi",
            RelocationKind::Addr16Ha => "Addr16Ha",
            RelocationKind::Addr14 => "Addr14",
            RelocationKind::Rel24 => "Rel24",
            RelocationKind::Rel14 => "Rel14",
        };

        println!(
            "  #{:<4} {:<20} target: [section: {:>2}, offset: {}], reference: [module: {:>4}, \
             section: {:>2}, offset: {}]",
            num(i as u32),
            kind,
            num(relocation.target.section),
            hex8(relocation.target.offset),
            num(relocation.module),
            num(relocation.reference.section),
            hex8(relocation.reference.offset)
        );
    }
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

fn output_sections_data<D: Deserializer + Seeker>(rel: &mut RelReader<D>, width: usize) {
    println!("sections:");
    for i in 0..rel.sections.len() {
        let section = rel.section_at(i).unwrap();
        println!("  [ section: {:>2} ]", num(i as u32));
        output_data(&section.1, width);
    }
}

fn output<D: Deserializer + Seeker>(
    rel: &mut RelReader<D>,
    dump_header: bool,
    dump_sections: bool,
    dump_imports: bool,
    dump_relocations: bool,
    dump_data: bool,
    width: usize,
) {
    if dump_header {
        output_header(rel);
    }

    if dump_sections {
        output_sections(rel);
    }

    if dump_imports {
        output_imports(rel);
    }

    if dump_relocations {
        output_relocations(rel);
    }

    if dump_data {
        output_sections_data(rel, width);
    }
}

fn main() {
    let args = Args::parse();

    let mut dump_header = args.header;
    let mut dump_sections = args.sections;
    let mut dump_imports = args.imports;
    let mut dump_relocations = args.relocations;
    let mut dump_data = args.data;
    let width = match args.width {
        0 => 1,
        _ => args.width,
    };

    if args.all {
        dump_header = true;
        dump_sections = true;
        dump_imports = true;
        dump_relocations = true;
        dump_data = true;
    }

    if !dump_header && !dump_sections && !dump_imports && !dump_relocations && !dump_data {
        println!("nothing to dump :(");
        return;
    }

    let file = std::fs::File::open(args.path).unwrap();
    let mut file = std::io::BufReader::new(file);

    if picori::compression::yaz0::is_yaz0(&mut file) {
        let mut reader = picori::compression::Yaz0Reader::new(file).unwrap();
        let decompressed = reader.decompress().unwrap();
        let mut cursor = Cursor::new(decompressed);
        let mut rel = picori::file::rel::RelReader::new(cursor).unwrap();
        output(
            &mut rel,
            dump_header,
            dump_sections,
            dump_imports,
            dump_relocations,
            dump_data,
            width,
        );
    } else {
        let mut rel = picori::file::rel::RelReader::new(file).unwrap();
        output(
            &mut rel,
            dump_header,
            dump_sections,
            dump_imports,
            dump_relocations,
            dump_data,
            width,
        );
    };
}
