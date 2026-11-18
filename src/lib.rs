#![forbid(unsafe_code)]

mod application;
pub mod mcp;
mod storage;
pub mod tui;

pub use application::{Application, Capabilities};
pub use storage::{CURRENT_SCHEMA_VERSION, StorageError};
