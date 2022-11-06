use std::path::PathBuf;

use clap::Parser;
use colored::{ColoredString, Colorize};
use picori::file::gcm;

extern crate picori;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    name = "gcm_dump",
    bin_name = "gcm_dump",
    author="Julgodis <self@julgodis.xyz>", 
    version=env!("CARGO_PKG_VERSION"), 
    about="Example program to dump .gcm/.iso files using picori", 
    long_about = None)]
struct Args {
    /// Path to the file to dump
    #[arg()]
    path: PathBuf,
    /// Dump boot.bin
    #[arg(long)]
    boot: bool,
    /// Dump bi2.bin
    #[arg(short = 'd', short, long)]
    bi2: bool,
    /// Dump apploader.img
    #[arg(short = 'l', long)]
    apploader: bool,
    /// Dump main.dol
    #[arg(short = 'e', long)]
    dol: bool,
    /// Dump fst.bin
    #[arg(short, long)]
    fst: bool,
    /// Dump data
    #[arg(short, long)]
    data: bool,
    /// Dump all
    #[arg(short, long)]
    all: bool,
    /// Column width
    #[arg(short, long, default_value = "32")]
    width: usize,
}

fn hex2(value: u8) -> ColoredString {
    format!("{:#04x}", value).cyan()
}

fn hex8(value: u32) -> ColoredString {
    format!("{:#010x}", value).cyan()
}

fn num(value: u32) -> ColoredString {
    format!("{}", value).cyan()
}

fn output_boot(boot: &gcm::Boot) {
    println!("boot.bin:");
    println!("  console id: {}", hex2(boot.console_id));
    println!(
        "  game code: {} {}",
        hex2(boot.game_code[0]),
        hex2(boot.game_code[1])
    );
    println!("  country code: {}", hex2(boot.country_code));
    println!(
        "  maker code: {} {}",
        hex2(boot.maker_code[0]),
        hex2(boot.maker_code[1])
    );
    println!("  disc id: {}", hex2(boot.disc_id));
    println!("  version: {}", hex2(boot.version));
    println!("  audio streaming: {}", hex2(boot.audio_streaming));
    println!(
        "  streaming buffer size: {}",
        hex2(boot.streaming_buffer_size)
    );
    println!("  magic: {}", hex8(boot.magic));
    println!(
        "  debug_monitor_offset: {}",
        hex8(boot.debug_monitor_offset)
    );
    println!(
        "  debug_monitor_address: {}",
        hex8(boot.debug_monitor_address)
    );
    println!(
        "  main_executable_offset: {}",
        hex8(boot.main_executable_offset)
    );
    println!("  fst_offset: {}", hex8(boot.fst_offset));
    println!("  fst_size: {}", hex8(boot.fst_size));
    println!("  fst_max_size: {}", hex8(boot.fst_max_size));
    println!("  user_position: {}", hex8(boot.user_position));
    println!("  user_length: {}", hex8(boot.user_length));
}

fn output_bi2(bi2: &gcm::Bi2) {
    println!("bi2.bin:");
    let mut options = bi2
        .options()
        .iter()
        .map(|x| (*x.0, *x.1))
        .collect::<Vec<_>>();

    options.sort_by(|a, b| a.0.cmp(&b.0));
    for (i, value) in options {
        println!("  [{:04x}]: {} ({})", i.index(), hex8(value), num(value));
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

fn output_apploader(apploader: &gcm::Apploader, data: bool, width: usize) {
    println!("apploader.img:");
    println!("  entry point: {}", hex8(apploader.entry_point));
    println!("  size: {}", hex8(apploader.size));
    println!("  trailer size: {}", hex8(apploader.trailer_size));
    println!("  unknown: {}", hex8(apploader.unknown));
    if data {
        println!("  data:");
        output_data(&apploader.data, width);
    }
}

fn main() {
    let args = Args::parse();

    let mut dump_boot = args.boot;
    let mut dump_bi2 = args.bi2;
    let mut dump_apploader = args.apploader;
    let mut dump_dol = args.dol;
    let mut dump_fst = args.fst;

    let data = args.data;
    let width = match args.width {
        0 => 1,
        _ => args.width,
    };

    if args.all {
        dump_boot = true;
        dump_bi2 = true;
        dump_apploader = true;
        dump_dol = true;
        dump_fst = true;
    }

    if !dump_boot && !dump_bi2 && !dump_apploader && !dump_dol && !dump_fst {
        println!("nothing to dump :(");
        return;
    }

    let file = std::fs::File::open(args.path).unwrap();
    let mut file = std::io::BufReader::new(file);
    let gcm = gcm::parse(&mut file).unwrap();

    if dump_boot {
        output_boot(&gcm.boot());
    }

    if dump_bi2 {
        output_bi2(&gcm.bi2());
    }

    if dump_apploader {
        output_apploader(&gcm.apploader(), data, width);
    }
}
