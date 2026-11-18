use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    path::Path,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::storage::{SqliteStore, StorageError};

#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct Capabilities {
    pub contract_version: String,
    pub database_schema_version: u32,
    pub interfaces: Vec<String>,
    pub available_tools: Vec<String>,
    pub unavailable_domains: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountKind {
    Bank,
    Card,
    Cash,
    Wallet,
    Marketplace,
}

impl AccountKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Bank => "bank",
            Self::Card => "card",
            Self::Cash => "cash",
            Self::Wallet => "wallet",
            Self::Marketplace => "marketplace",
        }
    }

    pub(crate) fn from_stored(value: &str) -> Option<Self> {
        match value {
            "bank" => Some(Self::Bank),
            "card" => Some(Self::Card),
            "cash" => Some(Self::Cash),
            "wallet" => Some(Self::Wallet),
            "marketplace" => Some(Self::Marketplace),
            _ => None,
        }
    }
}

impl Display for AccountKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
    pub archived: bool,
    pub revision: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct CreateAccount {
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
}

#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct AccountList {
    pub accounts: Vec<Account>,
}

#[derive(Debug)]
pub enum ApplicationError {
    Validation {
        field: &'static str,
        message: &'static str,
    },
    Conflict {
        message: String,
    },
    Storage(StorageError),
}

impl ApplicationError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Validation { .. } => "validation_failed",
            Self::Conflict { .. } => "conflict",
            Self::Storage(_) => "storage_failed",
        }
    }

    #[must_use]
    pub const fn field(&self) -> Option<&'static str> {
        match self {
            Self::Validation { field, .. } => Some(field),
            Self::Conflict { .. } | Self::Storage(_) => None,
        }
    }
}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation { field, message } => write!(formatter, "{field}: {message}"),
            Self::Conflict { message } => formatter.write_str(message),
            Self::Storage(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Validation { .. } | Self::Conflict { .. } => None,
        }
    }
}

impl From<StorageError> for ApplicationError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
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
            available_tools: vec![
                "get_capabilities".to_owned(),
                "list_accounts".to_owned(),
                "create_account".to_owned(),
            ],
            unavailable_domains: vec![
                "account_update_archive_restore".to_owned(),
                "categories".to_owned(),
                "transactions".to_owned(),
                "transfers".to_owned(),
                "budgets".to_owned(),
                "statistics".to_owned(),
                "imports".to_owned(),
            ],
        })
    }

    /// Returns active accounts in normalized-name order.
    ///
    /// # Errors
    ///
    /// Returns an error when the database cannot be read.
    pub fn list_accounts(&self) -> Result<AccountList, StorageError> {
        Ok(AccountList {
            accounts: self.store.list_accounts()?,
        })
    }

    /// Validates and creates one active account.
    ///
    /// # Errors
    ///
    /// Returns a validation error for an invalid name or currency, a conflict for a
    /// duplicate normalized name, or a storage error when the write cannot complete.
    pub fn create_account(&self, input: &CreateAccount) -> Result<Account, ApplicationError> {
        let name = input.name.split_whitespace().collect::<Vec<_>>().join(" ");
        if name.is_empty() {
            return Err(ApplicationError::Validation {
                field: "name",
                message: "enter an account name",
            });
        }
        if name.chars().count() > 80 {
            return Err(ApplicationError::Validation {
                field: "name",
                message: "use at most 80 characters",
            });
        }
        if name.chars().any(char::is_control) {
            return Err(ApplicationError::Validation {
                field: "name",
                message: "control characters are not allowed",
            });
        }

        let currency = input.currency.trim().to_ascii_uppercase();
        if currency.len() != 3 || !currency.bytes().all(|byte| byte.is_ascii_alphabetic()) {
            return Err(ApplicationError::Validation {
                field: "currency",
                message: "use a three-letter currency code",
            });
        }

        // shortcut: names collapse whitespace and lowercase but do not NFC-normalize; add NFC before a stable release.
        let name_key = name.to_lowercase();
        self.store
            .create_account(&name, &name_key, input.kind, &currency)
            .map_err(|error| match error {
                StorageError::DuplicateAccountName => ApplicationError::Conflict {
                    message: format!("an account named '{name}' already exists"),
                },
                other => ApplicationError::Storage(other),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_current_foundation_tools() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let app = Application::open(directory.path().join("finance.db")).expect("application");

        let capabilities = app.capabilities().expect("capabilities");

        assert_eq!(capabilities.database_schema_version, 2);
        assert_eq!(
            capabilities.available_tools,
            ["get_capabilities", "list_accounts", "create_account"]
        );
        assert!(
            capabilities
                .unavailable_domains
                .contains(&"account_update_archive_restore".to_owned())
        );
    }

    #[test]
    fn creates_lists_and_rejects_a_normalized_duplicate() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("finance.db");
        let app = Application::open(&path).expect("application");

        let account = app
            .create_account(&CreateAccount {
                name: "  Daily   Bank  ".to_owned(),
                kind: AccountKind::Bank,
                currency: "sgd".to_owned(),
            })
            .expect("account");

        assert_eq!(account.name, "Daily Bank");
        assert_eq!(account.currency, "SGD");
        assert_eq!(account.kind, AccountKind::Bank);
        drop(app);

        let reopened = Application::open(path).expect("reopened application");
        assert_eq!(
            reopened.list_accounts().expect("accounts").accounts,
            [account]
        );
        let duplicate = reopened
            .create_account(&CreateAccount {
                name: "daily bank".to_owned(),
                kind: AccountKind::Cash,
                currency: "SGD".to_owned(),
            })
            .expect_err("normalized duplicate");
        assert_eq!(duplicate.code(), "conflict");
    }

    #[test]
    fn rejects_an_invalid_currency_before_storage() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let app = Application::open(directory.path().join("finance.db")).expect("application");

        let error = app
            .create_account(&CreateAccount {
                name: "Cash".to_owned(),
                kind: AccountKind::Cash,
                currency: "dollars".to_owned(),
            })
            .expect_err("invalid currency");

        assert_eq!(error.code(), "validation_failed");
        assert_eq!(error.field(), Some("currency"));
        assert!(app.list_accounts().expect("accounts").accounts.is_empty());
    }

    #[test]
    fn rejects_control_characters_in_account_names() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let app = Application::open(directory.path().join("finance.db")).expect("application");

        let error = app
            .create_account(&CreateAccount {
                name: "Unsafe\u{1b}Name".to_owned(),
                kind: AccountKind::Wallet,
                currency: "SGD".to_owned(),
            })
            .expect_err("control character");

        assert_eq!(error.code(), "validation_failed");
        assert_eq!(error.field(), Some("name"));
    }
}
