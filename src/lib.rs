//! rusync
//!
//! Implements copy from one directory to an other
//!
//! To use rusync as a library, start with the [Syncer](sync/struct.Syncer.html) struct.
//!
//! To customize its output, implement the [ProgressInfo](progress/trait.ProgressInfo.html) trait.

extern crate colored;
extern crate filetime;
extern crate term_size;

pub mod console_info;
mod entry;
mod fsops;
pub mod progress;
pub mod sync;
mod workers;
pub use sync::Syncer;
pub use sync::Stats;
