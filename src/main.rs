#[macro_use]
extern crate structopt;
extern crate colored;
extern crate rusync;

use colored::Colorize;
use rusync::Stats;
use rusync::Syncer;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rusync")]
struct Opt {
    #[structopt(long = "no-perms")]
    no_preserve_permissions: bool,

    #[structopt(parse(from_os_str))]
    source: PathBuf,

    #[structopt(parse(from_os_str))]
    destination: PathBuf,
}

impl Opt {
    fn preserve_permissions(&self) -> bool {
        !self.no_preserve_permissions
    }
}

fn print_stats(stats: &Stats) {
    println!(
        "{} Synced {} files ({} up to date)",
        " ✓".color("green"),
        stats.num_synced,
        stats.up_to_date
    );
    println!(
        "{} files copied, {} symlinks created, {} symlinks updated",
        stats.copied, stats.symlink_created, stats.symlink_updated
    );
}

fn main() {
    let opt = Opt::from_args();
    let source = &opt.source;
    if !source.is_dir() {
        eprintln!("{} is not a directory", source.to_string_lossy());
        process::exit(1);
    }
    let destination = &opt.destination;
    let preserve_permissions = opt.preserve_permissions();
    println!(
        "{} Syncing from {} to {} …",
        "::".color("blue"),
        source.to_string_lossy().bold(),
        destination.to_string_lossy().bold()
    );

    let mut syncer = Syncer::new(&source, &destination);
    syncer.preserve_permissions(preserve_permissions);
    let stats = syncer.sync();
    match stats {
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
        Ok(stats) => {
            print_stats(&stats);
            process::exit(0);
        }
    }
}
