use std::path::{PathBuf, Path};

use clap::Parser;
use picori::{Rarc};

extern crate picori;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    name = "rarc_dump",
    bin_name = "rarc_dump",
    author="Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>", 
    version=env!("CARGO_PKG_VERSION"), 
    about="Example program to dump .rarc files using picori", 
    long_about = None)]
struct Args {
    /// Path to the file to dump
    #[arg()]
    path:     PathBuf,
    /// Where to dump files
    #[arg(short, long)]
    output:   Option<PathBuf>,
}

fn visit_dirs(rarc: &mut Rarc<&mut std::io::BufReader<std::fs::File>>, dir: &Path, extract: &Option<PathBuf>) -> picori::Result<()> {
    let dir = dir.into();
    if let Some(path) = extract {
        std::fs::create_dir_all(path.join(dir))?;
    }
    for entry in rarc.read_dir(dir)? {
        let filename = entry.name(rarc)?;
        if filename == "." || filename == ".." {
            continue;
        }
        if entry.is_dir(rarc) {
            visit_dirs(rarc, &dir.join(filename), extract)?;
        } else if let Some(path) = extract {
            let data = entry.data(rarc)?;
            std::fs::write(path.join(dir).join(filename), data)?;
        }
    }
    Ok(())
}

fn main() {
    let args = Args::parse();

    let file = std::fs::File::open(args.path).unwrap();
    let mut file = std::io::BufReader::new(file);
    let mut rarc = Rarc::new(&mut file).unwrap();

    visit_dirs(&mut rarc, Path::new(""), &args.output).unwrap();
}
