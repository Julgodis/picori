use clap::Parser;
use picori::rel::ImportKind;
use picori::Rel;
use std::path::PathBuf;

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
    path: PathBuf,
    /// Dump header
    #[arg(short = 't', long)]
    header: bool,
    /// Dump sections
    #[arg(short, long)]
    sections: bool,
    /// Dump imports
    #[arg(short, long)]
    imports: bool,
    /// Dump relocations
    #[arg(short, long)]
    relocations: bool,
    /// Dump section data
    #[arg(short, long)]
    data: bool,
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

fn num(value: u32) -> String {
    format!("\x1b[36m{}\x1b[0m", value)
}

fn output_header(rel: &Rel) {
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

fn output_sections(rel: &Rel) {
    println!("sections:");
    for (i, section) in rel.sections.iter().enumerate() {
        println!(
            "  #{:<2} offset: {}, size: {}{}{}{}{}",
            num(i as u32),
            hex8(section.offset),
            hex8(section.size),
            if section.executable {
                "\x1b[31m [executable]\x1b[0m"
            } else {
                ""
            },
            if section.unknown {
                "\x1b[31m [unknown]\x1b[0m"
            } else {
                ""
            },
            if i == 0 {
                "\x1b[32m [null section]\x1b[0m"
            } else {
                ""
            },
            if i != 0 && section.offset == 0 && section.size == 0 {
                "\x1b[32m [unused]\x1b[0m"
            } else {
                ""
            }
        );
    }
}

fn import_kind_to_string(kind: ImportKind) -> &'static str {
    match kind {
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
    }
}

fn output_imports(rel: &Rel) {
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
            println!(
                "  #{:<4} {:<20} section: {:>2}, offset: {}, addend: {}",
                num(j as u32),
                import_kind_to_string(import.kind),
                num(import.section as u32),
                hex4(import.offset),
                hex8(import.addend)
            );
        }
    }
}

fn output_relocations(rel: &Rel) {
    println!("relocation:");

    for (i, relocation) in rel.relocations().enumerate() {
        println!(
            "  #{:<4} {:<20} target: [section: {:>2}, offset: {}], reference: [module: {:>4}, \
             section: {:>2}, offset: {}]",
            num(i as u32),
            import_kind_to_string(relocation.kind),
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

fn output_sections_data(rel: &Rel, width: usize) {
    println!("sections:");
    for (i, section) in rel.sections.iter().enumerate() {
        println!("  [ section: {:>2} ]", num(i as u32));
        output_data(&section.data, width);
    }
}

fn output(
    rel: &Rel,
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

fn main() -> picori::Result<()> {
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
        return Ok(());
    }

    let file = std::fs::File::open(args.path)?;
    let file = std::io::BufReader::new(file);
    let file = picori::Yaz0Reader::new(file)?;
    let rel = Rel::from_binary(file)?;

    output(
        &rel,
        dump_header,
        dump_sections,
        dump_imports,
        dump_relocations,
        dump_data,
        width,
    );

    Ok(())
}
