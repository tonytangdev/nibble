//! Implementations of `nibble-core`'s ports against the real world:
//! local log files, price catalogs and plan detection.

mod catalog;
mod claude_code;
mod history;

pub use catalog::JsonCatalog;
pub use claude_code::{ClaudeCode, PROVIDER as CLAUDE_PROVIDER};
