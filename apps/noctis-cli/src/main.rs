use std::{env, fs, process};
use noctis_html::parse;

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("Usage: noctis-cli <document.html>");
        process::exit(2);
    };

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("noctis-cli: cannot read {path}: {error}");
        process::exit(1);
    });

    let document = parse(&source);
    print!("{}", document.dump());
}
