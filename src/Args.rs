#[derive(Debug)]
pub struct Args {
    pub make: bool,
    pub archiveName: String,
    pub extract: bool,
    pub archivePath: String,
    pub extractPath: String,
    pub elements: Vec<String>,
    pub verbose: bool,
    pub help: bool,
    pub version: bool,
    pub inspect: bool
}

impl Args {
    fn new() -> Self {
        Self {
            make: false,
            archiveName: String::new(),
            extract: false,
            archivePath: String::new(),
            extractPath: String::new(),
            elements: Vec::<String>::new(),
            verbose: false,
            help: false,
            version: false,
            inspect: false
        }
    }

    pub fn parse(stdargs: Vec<String>) -> Args {
        let mut args = Args::new();
        let mut index: usize = 0;

        while index < stdargs.len() {
            let chunk = stdargs.get(index).unwrap().as_str();

            if chunk == "-m" || chunk == "--make" {
                index += 1;
                args.make = true;

                if index >= stdargs.len() {
                    eprintln!("Error: expected archive name after `-m | --make` flag");
                    std::process::exit(1);
                }

                args.archiveName = stdargs.get(index).unwrap().to_string();

            } else if chunk == "-e" || chunk == "--extract" {
                index += 1;
                args.extract = true;

                if index >= stdargs.len() {
                    eprintln!("Error: expected extraction path after `-e | --extract` flag");
                    std::process::exit(1);
                }

                args.archivePath = stdargs.get(index).unwrap().to_string();

            } else if chunk == "-v" || chunk == "--verbose" {
                args.verbose = true;

            } else if chunk == "-h" || chunk == "--help" {
                args.help = true;

            } else if chunk == "-V" || chunk == "--version" {
                args.version = true;

            } else if chunk == "-i" || chunk == "--inspect" {
                args.inspect = true;
                index += 1;

                if index >= stdargs.len() {
                    eprintln!("Error: expected archive path after `-i | --inspect` flag");
                    std::process::exit(1);
                }

                args.archivePath = stdargs.get(index).unwrap().to_string();

            } else {
                if args.extract {
                    args.extractPath = chunk.to_string()

                } else {
                    args.elements.push(chunk.to_string());
                }
            }

            index += 1;
        }

        if args.extractPath.is_empty() {
            args.extractPath = String::from(".");
        }

        return args;
    }
}