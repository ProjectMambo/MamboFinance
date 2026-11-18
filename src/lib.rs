#![forbid(unsafe_code)]

mod application;
pub mod mcp;
mod storage;
pub mod tui;

pub use application::{
    Account, AccountKind, AccountList, Application, ApplicationError, Capabilities, CreateAccount,
};
pub use storage::{CURRENT_SCHEMA_VERSION, StorageError};
