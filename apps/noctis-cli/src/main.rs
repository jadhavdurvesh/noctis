use noctis_css::parse_stylesheet;
use noctis_html::{parse_document, parse_fragment};
use std::{env, fs, process};

#[derive(Clone, Copy)]
enum OutputMode {
    Tree,
    Html,
    Css,
}

fn usage() {
    println!(
        "NOCTIS Engine Lab 0.1.0\n\
         Usage: noctis-cli [OPTIONS] <file>\n\
         \n\
         Options:\n\
           --tree       Print the parsed DOM tree (default)\n\
           --html       Print normalized HTML serialization\n\
           --css        Parse and inspect a CSS stylesheet\n\
           --fragment   Parse an HTML fragment instead of a full document\n\
           -h, --help   Show this help\n\
           -V, --version Show version information"
    );
}

fn main() {
    let mut mode = OutputMode::Tree;
    let mut fragment = false;
    let mut path: Option<String> = None;

    for argument in env::args().skip(1) {
        match argument.as_str() {
            "--tree" => mode = OutputMode::Tree,
            "--html" => mode = OutputMode::Html,
            "--css" => mode = OutputMode::Css,
            "--fragment" => fragment = true,
            "-h" | "--help" => {
                usage();
                return;
            }
            "-V" | "--version" => {
                println!("NOCTIS Engine Lab 0.1.0");
                return;
            }
            value if value.starts_with('-') => {
                eprintln!("noctis-cli: unknown option: {value}");
                eprintln!("Try 'noctis-cli --help' for usage.");
                process::exit(2);
            }
            value => {
                if path.replace(value.to_owned()).is_some() {
                    eprintln!("noctis-cli: only one input file can be supplied");
                    process::exit(2);
                }
            }
        }
    }

    let Some(path) = path else {
        eprintln!("noctis-cli: an input file is required");
        eprintln!("Try 'noctis-cli --help' for usage.");
        process::exit(2);
    };

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("noctis-cli: cannot read {path}: {error}");
        process::exit(1);
    });

    match mode {
        OutputMode::Css => {
            let rules = parse_stylesheet(&source);
            println!("{rules:#?}");
        }
        OutputMode::Tree | OutputMode::Html => {
            let document = if fragment {
                parse_fragment(&source)
            } else {
                parse_document(&source)
            };
            match mode {
                OutputMode::Tree => print!("{}", document.dump()),
                OutputMode::Html => println!("{}", document.to_html()),
                OutputMode::Css => unreachable!(),
            }
        }
    }
}
