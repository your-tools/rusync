extern crate colored;

mod app;

use std::env;
use std::process;
use std::path::PathBuf;


fn parse_args() -> Result<(PathBuf, PathBuf), String> {
    let args: Vec<String> = env::args().collect();
    let arg_count = args.len();
    if arg_count < 3 {
        return Err(format!("Usage: {} SRC DEST", args[0]))
    }
    let source = PathBuf::from(&args[1]);
    let destination = PathBuf::from(&args[2]);
    Ok((source, destination))
}


fn main() {
    let parsed = parse_args();
    if let Err(err) = parsed {
            eprintln!("{}", err);
            process::exit(1)
    }
    let (source, destination) = parsed.unwrap();

    if let Err(err) = app::sync(source, destination) {
        eprintln!("{}", err);
        process::exit(1);
    }
    process::exit(0);
}
