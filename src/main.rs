#[macro_use]
extern crate structopt;
extern crate colored;
extern crate rusync;

use colored::Colorize;
use rusync::pipeline::Pipeline;
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

    let pipeline = Pipeline::new(&source, &destination);
    //pipeline.preserve_permissions(preserve_permissions);
    let stats = pipeline.run();
    match stats {
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
        Ok(stats) => {
            println!(
                "{} Synced {} files ({} up to date)",
                " ✓".color("green"),
                stats.total,
                stats.up_to_date
            );
            println!(
                "{} files copied, {} symlinks created, {} symlinks updated",
                stats.copied, stats.symlink_created, stats.symlink_updated
            );

            process::exit(0);
        }
    }
}
