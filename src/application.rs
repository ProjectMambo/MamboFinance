use std::path::Path;

use schemars::JsonSchema;
use serde::Serialize;

use crate::storage::{SqliteStore, StorageError};

#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct Capabilities {
    pub contract_version: String,
    pub database_schema_version: u32,
    pub interfaces: Vec<String>,
    pub available_tools: Vec<String>,
    pub unavailable_domains: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Application {
    store: SqliteStore,
}

impl Application {
    /// Opens the application with its platform user-data database.
    ///
    /// # Errors
    ///
    /// Returns an error when the data directory, database, or migration is unavailable.
    pub fn open_default() -> Result<Self, StorageError> {
        Ok(Self {
            store: SqliteStore::open_default()?,
        })
    }

    /// Opens the application with an explicit database path.
    ///
    /// # Errors
    ///
    /// Returns an error when the directory, database, or migration cannot be prepared.
    pub fn open(database_path: impl AsRef<Path>) -> Result<Self, StorageError> {
        Ok(Self {
            store: SqliteStore::open(database_path)?,
        })
    }

    /// Returns the currently implemented interface and storage contract.
    ///
    /// # Errors
    ///
    /// Returns an error when the database schema version cannot be read.
    pub fn capabilities(&self) -> Result<Capabilities, StorageError> {
        Ok(Capabilities {
            contract_version: "0.1".to_owned(),
            database_schema_version: self.store.schema_version()?,
            interfaces: vec!["mamboui_tui".to_owned(), "mcp_streamable_http".to_owned()],
            available_tools: vec!["get_capabilities".to_owned()],
            unavailable_domains: vec![
                "accounts".to_owned(),
                "categories".to_owned(),
                "transactions".to_owned(),
                "transfers".to_owned(),
                "budgets".to_owned(),
                "statistics".to_owned(),
                "imports".to_owned(),
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_only_the_foundation_tool() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let app = Application::open(directory.path().join("finance.db")).expect("application");

        let capabilities = app.capabilities().expect("capabilities");

        assert_eq!(capabilities.database_schema_version, 1);
        assert_eq!(capabilities.available_tools, ["get_capabilities"]);
        assert!(
            capabilities
                .unavailable_domains
                .contains(&"accounts".to_owned())
        );
    }
}
