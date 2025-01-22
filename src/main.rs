#![allow(non_snake_case)]
use crate::Archive::{extractArchive, makeArchive};

mod Args;
mod Archive;

const VERSION: &str = "1.0.0";

fn main() {
    let mut stdargs = std::env::args().collect::<Vec<String>>();
    stdargs.remove(0);

    let args = Args::Args::parse(stdargs);

    if args.help {
        println!("ezark: Easy Archiver");
        println!("usage: ezark ((-m | --make ARCHIVE) | (-e | --extract ARCHIVE)) [FLAGS] elements");
        println!("flags:");
        println!("    -m | --make ARCHIVE       Create archive with provided name");
        println!("    -e | --extract ARCHIVE    Extract specified archive");
        println!("    -v | --verbose            Print each step");
        println!("    -h | --help               Show this message and exit");
        println!();

    } else if args.version {
        println!("ezark version: {}", VERSION);

    } else if args.make {
        makeArchive(args.elements, args.archiveName, args.verbose);

    } else if args.extract {
        extractArchive(args.archivePath, args.extractPath, args.verbose);
    }
}