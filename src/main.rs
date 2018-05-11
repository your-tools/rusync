extern crate colored;
extern crate rusync;

use colored::Colorize;
use rusync::sync;
use std::env;
use std::path::PathBuf;
use std::process;

fn parse_args() -> Result<(PathBuf, PathBuf), String> {
    let args: Vec<String> = env::args().collect();
    let arg_count = args.len();
    if arg_count < 3 {
        return Err(format!("Usage: {} SRC DEST", args[0]));
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
    println!(
        "{} Syncing from {} to {} …",
        "::".color("blue"),
        source.to_string_lossy().bold(),
        destination.to_string_lossy().bold()
    );

    let outcome = sync::sync(&source, &destination);
    match outcome {
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
        Ok(stats) => {
            let total = stats.total;
            let up_to_date = stats.up_to_date;
            let copied = stats.copied;
            let symlink_created = stats.symlink_created;
            let symlink_updated = stats.symlink_updated;

            println!(
                "{} Synced {} files ({} up to date)",
                " ✓".color("green"),
                total,
                up_to_date
            );
            println!(
                "{} files copied, {} symlinks created, {} symlinks updated",
                copied, symlink_created, symlink_updated
            );

            process::exit(0);
        }
    }
}
