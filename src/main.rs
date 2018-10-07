#[macro_use]
extern crate structopt;
extern crate colored;
extern crate rusync;

use rusync::console_info::ConsoleProgressInfo;
use rusync::sync::SyncOptions;
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

fn main() {
    let opt = Opt::from_args();
    let source = &opt.source;
    if !source.is_dir() {
        eprintln!("{} is not a directory", source.to_string_lossy());
        process::exit(1);
    }
    let destination = &opt.destination;

    let console_info = ConsoleProgressInfo::new();
    let mut options = SyncOptions::new();
    options.preserve_permissions = opt.preserve_permissions();
    let syncer = Syncer::new(&source, &destination, options, Box::new(console_info));
    let stats = syncer.sync();
    match stats {
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
        Ok(_) => {
            process::exit(0);
        }
    }
}
