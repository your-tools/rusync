#![deny(warnings)]

extern crate colored;
extern crate filetime;
extern crate terminal_size;

mod entry;
mod fsops;
mod progress;
pub mod sync;
mod workers;
pub use sync::Stats;
pub use sync::Syncer;
