extern crate colored;
extern crate filetime;

mod entry;
mod fsops;
mod progress;
pub mod sync;
mod workers;
pub use sync::Syncer;
