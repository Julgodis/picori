use std::path::PathBuf;

use clap::Parser;
use picori::Dol;

extern crate picori;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    name = "dol_dump",
    bin_name = "dol_dump",
    author="Julgodis <self@julgodis.xyz>", 
    version=env!("CARGO_PKG_VERSION"), 
    about="Example program to dump .dol files using picori", 
    long_about = None)]
struct Args {
    /// Path to the file to dump
    #[arg()]
    path:     PathBuf,
    /// Dump header
    #[arg(short = 't', long)]
    header:   bool,
    /// Dump sections
    #[arg(short, long)]
    sections: bool,
    /// Dump data
    #[arg(short, long)]
    data:     bool,
    /// Dump all
    #[arg(short, long)]
    all:      bool,
    /// Column width
    #[arg(short, long, default_value = "32")]
    width:    usize,
}

fn main() {
    let args = Args::parse();

    let mut dump_header = args.header;
    let mut dump_sections = args.sections;
    let mut dump_data = args.data;

    if args.all {
        dump_header = true;
        dump_sections = true;
        dump_data = true;
    }

    if !dump_header && !dump_sections && !dump_data {
        println!("nothing to dump :(");
        return;
    }

    let file = std::fs::File::open(args.path).unwrap();
    let mut file = std::io::BufReader::new(file);
    let dol = Dol::from_binary(&mut file).unwrap();

    if dump_header {
        println!("header:");
        for i in 0..7 {
            println!(
                "  [{:>2}] text  offset: 0x{:08x}, address: 0x{:08x}, size: 0x{:08x}",
                i, dol.header.text_offset[i], dol.header.text_address[i], dol.header.text_size[i]
            );
        }
        for i in 0..11 {
            println!(
                "  [{:>2}] data  offset: 0x{:08x}, address: 0x{:08x}, size: 0x{:08x}",
                i, dol.header.data_offset[i], dol.header.data_address[i], dol.header.data_size[i]
            );
        }
        println!(
            "  bss address: 0x{:08x}, size: 0x{:08x}",
            dol.header.bss_address, dol.header.bss_size
        );
        println!("  entry point: 0x{:08x}", dol.header.entry_point);
    }

    if dump_sections {
        println!("sections:");
        for (i, section) in dol.sections.iter().enumerate() {
            println!(
                "  [{:>2}] {:<15} address: 0x{:08x}, size: 0x{:06x} (0x{:06x})",
                i, section.name, section.address, section.size, section.aligned_size
            );
        }
    }

    if dump_data {
        let width = match args.width {
            0 => 1,
            _ => args.width,
        };

        println!("data:");
        for (i, section) in dol.sections.iter().enumerate() {
            println!(
                "  [{:>2}] {:<15} address: 0x{:08x}",
                i, section.name, section.address
            );
            for (j, line) in section.data.chunks(width).enumerate() {
                print!("{:06x}: ", j * 32);
                for byte in line {
                    print!("{:02x} ", byte);
                }
                println!();
            }
        }
    }
}
