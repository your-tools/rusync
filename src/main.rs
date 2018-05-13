#[macro_use]
extern crate structopt;
extern crate colored;
extern crate rusync;

use colored::Colorize;
use rusync::sync;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;
use sync::Syncer;

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

fn main() {
    let opt = Opt::from_args();
    let source = &opt.source;
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
    let outcome = syncer.sync();
    match outcome {
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
        Ok(_) => {
            let stats = syncer.stats();
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
