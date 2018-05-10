extern crate colored;

mod sync;

use std::env;
use std::process;
use std::path::PathBuf;
use colored::Colorize;


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
    println!("{} Syncing from {} to {} …",
             "::".color("blue"),
             source.to_string_lossy().bold(),
             destination.to_string_lossy().bold()
    );


    let outcome = sync::sync(&source, &destination);
    match outcome {
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        },
        Ok(stats) => {
            println!("{} Synced {} files ({} up to date)", " ✓".color("green"), stats.copied, stats.up_to_date);
            process::exit(0);
        }
    }
}
