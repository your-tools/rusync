extern crate colored;
extern crate filetime;
extern crate term_size;

pub mod console_info;
mod entry;
mod fsops;
pub mod progress;
pub mod sync;
mod workers;
pub use sync::Stats;
pub use sync::Syncer;
