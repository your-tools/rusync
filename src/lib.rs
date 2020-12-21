//! rusync
//!
//! Implements copy from one directory to an other
//!
//! To use rusync as a library, start with the [Syncer](sync/struct.Syncer.html) struct.
//!
//! To customize its output, implement the [ProgressInfo](progress/trait.ProgressInfo.html) trait.

//! # Example
//!
//! ```
//! let console_info = rusync::ConsoleProgressInfo::new();
//! // or any struct that implements the ProgressInfo trait
//!
//! let options = rusync::SyncOptions::default();
//!
//!
//! let source = std::path::Path::new("src");
//! let destination = std::path::Path::new("dest");
//! let syncer = rusync::Syncer::new(&source, &destination, options, Box::new(console_info));
//! let stats = syncer.sync();
//! match stats {
//!     Err(err) => {
//!         eprintln!("Error when syncing: {}", err);
//!     }
//!     Ok(stats) => {
//!         println!("Transfered {} files", stats.copied);
//!     }
//! }
//! ```
//!
pub mod console_info;
mod entry;
mod fsops;
pub mod progress;
pub mod sync;
mod workers;
pub use crate::console_info::ConsoleProgressInfo;
pub use crate::sync::Stats;
pub use crate::sync::SyncOptions;
pub use crate::sync::Syncer;
