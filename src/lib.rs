extern crate colored;
extern crate filetime;

mod entry;
mod fsops;
pub mod sync;
pub use sync::Syncer;
