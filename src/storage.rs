use std::{
    env,
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::{Account, AccountKind};

pub const CURRENT_SCHEMA_VERSION: u32 = 2;

#[derive(Debug)]
pub enum StorageError {
    DataDirectoryUnavailable,
    DuplicateAccountName,
    Io(std::io::Error),
    Database(rusqlite::Error),
    UnsupportedSchema { found: u32, supported: u32 },
}

impl Display for StorageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataDirectoryUnavailable => {
                formatter.write_str("the platform user-data directory is unavailable")
            }
            Self::DuplicateAccountName => {
                formatter.write_str("an account with that normalized name already exists")
            }
            Self::Io(error) => write!(formatter, "storage I/O failed: {error}"),
            Self::Database(error) => write!(formatter, "SQLite operation failed: {error}"),
            Self::UnsupportedSchema { found, supported } => write!(
                formatter,
                "database schema version {found} is newer than supported version {supported}"
            ),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Database(error) => Some(error),
            Self::DataDirectoryUnavailable
            | Self::DuplicateAccountName
            | Self::UnsupportedSchema { .. } => None,
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SqliteStore {
    path: PathBuf,
}

impl SqliteStore {
    pub(crate) fn open_default() -> Result<Self, StorageError> {
        let path = default_database_path()?;
        if let Some(parent) = path.parent() {
            create_private_directory(parent)?;
        }
        Self::open_initialized(path)
    }

    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Self::open_initialized(path)
    }

    fn open_initialized(path: PathBuf) -> Result<Self, StorageError> {
        let store = Self { path };
        store.migrate()?;
        Ok(store)
    }

    pub(crate) fn schema_version(&self) -> Result<u32, StorageError> {
        let connection = self.connection()?;
        read_schema_version(&connection)
    }

    pub(crate) fn list_accounts(&self) -> Result<Vec<Account>, StorageError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, name, kind, currency, archived_at IS NOT NULL, revision,
                    created_at, updated_at
             FROM accounts
             WHERE archived_at IS NULL
             ORDER BY name_key, id",
        )?;
        let accounts = statement
            .query_map([], account_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(accounts)
    }

    pub(crate) fn create_account(
        &self,
        name: &str,
        name_key: &str,
        kind: AccountKind,
        currency: &str,
    ) -> Result<Account, StorageError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction
            .execute(
                "INSERT INTO accounts (name, name_key, kind, currency) VALUES (?1, ?2, ?3, ?4)",
                params![name, name_key, kind.as_str(), currency],
            )
            .map_err(map_account_insert_error)?;
        let id = transaction.last_insert_rowid();
        let account = transaction.query_row(
            "SELECT id, name, kind, currency, archived_at IS NOT NULL, revision,
                    created_at, updated_at
             FROM accounts
             WHERE id = ?1",
            [id],
            account_from_row,
        )?;
        transaction.commit()?;
        Ok(account)
    }

    fn connection(&self) -> Result<Connection, StorageError> {
        let connection = Connection::open(&self.path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        Ok(connection)
    }

    fn migrate(&self) -> Result<(), StorageError> {
        let mut connection = self.connection()?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );",
        )?;

        let version = read_schema_version(&connection)?;
        if version > CURRENT_SCHEMA_VERSION {
            return Err(StorageError::UnsupportedSchema {
                found: version,
                supported: CURRENT_SCHEMA_VERSION,
            });
        }
        for next_version in (version + 1)..=CURRENT_SCHEMA_VERSION {
            apply_migration(&mut connection, next_version)?;
        }
        Ok(())
    }
}

fn apply_migration(connection: &mut Connection, version: u32) -> Result<(), StorageError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    match version {
        1 => {}
        2 => transaction.execute_batch(
            "CREATE TABLE accounts (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL CHECK (name <> ''),
                name_key TEXT NOT NULL UNIQUE CHECK (name_key <> ''),
                kind TEXT NOT NULL CHECK (
                    kind IN ('bank', 'card', 'cash', 'wallet', 'marketplace')
                ),
                currency TEXT NOT NULL CHECK (currency GLOB '[A-Z][A-Z][A-Z]'),
                archived_at TEXT,
                revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
                created_at TEXT NOT NULL DEFAULT (
                    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                ),
                updated_at TEXT NOT NULL DEFAULT (
                    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                )
            );",
        )?,
        _ => unreachable!("migration version is bounded by CURRENT_SCHEMA_VERSION"),
    }
    transaction.execute(
        "INSERT INTO schema_migrations (version) VALUES (?1)",
        [version],
    )?;
    transaction.commit()?;
    Ok(())
}

fn account_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Account> {
    let kind = row.get::<_, String>(2)?;
    let kind = AccountKind::from_stored(&kind).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid stored account kind",
            )
            .into(),
        )
    })?;
    Ok(Account {
        id: row.get::<_, i64>(0)?.to_string(),
        name: row.get(1)?,
        kind,
        currency: row.get(3)?,
        archived: row.get(4)?,
        revision: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_account_insert_error(error: rusqlite::Error) -> StorageError {
    if matches!(
        error,
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                extended_code: rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE,
                ..
            },
            _
        )
    ) {
        StorageError::DuplicateAccountName
    } else {
        StorageError::Database(error)
    }
}

fn read_schema_version(connection: &Connection) -> Result<u32, StorageError> {
    let version = connection
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get::<_, Option<u32>>(0)
        })
        .optional()?
        .flatten()
        .unwrap_or(0);
    Ok(version)
}

fn default_database_path() -> Result<PathBuf, StorageError> {
    #[cfg(target_os = "windows")]
    let root = env::var_os("LOCALAPPDATA")
        .or_else(|| env::var_os("APPDATA"))
        .map(PathBuf::from);

    #[cfg(target_os = "macos")]
    let root = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library").join("Application Support"));

    #[cfg(all(unix, not(target_os = "macos")))]
    let root = env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| {
        env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".local").join("share"))
    });

    #[cfg(not(any(unix, target_os = "windows")))]
    let root: Option<PathBuf> = None;

    root.map(|path| path.join("mambofinance").join("mambofinance.db"))
        .ok_or(StorageError::DataDirectoryUnavailable)
}

fn create_private_directory(path: &Path) -> Result<(), StorageError> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_durable_and_repeatable() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("finance.db");

        let first = SqliteStore::open(&path).expect("first open");
        assert_eq!(first.schema_version().expect("schema version"), 2);
        let connection = first.connection().expect("connection");
        let versions = connection
            .prepare("SELECT version FROM schema_migrations ORDER BY version")
            .expect("statement")
            .query_map([], |row| row.get::<_, u32>(0))
            .expect("versions")
            .collect::<Result<Vec<_>, _>>()
            .expect("version rows");
        assert_eq!(versions, [1, 2]);
        drop(first);

        let reopened = SqliteStore::open(&path).expect("reopen");
        assert_eq!(reopened.schema_version().expect("schema version"), 2);
    }

    #[test]
    fn newer_schema_is_rejected() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("finance.db");
        let store = SqliteStore::open(&path).expect("store");
        let connection = store.connection().expect("connection");
        connection
            .execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                params![CURRENT_SCHEMA_VERSION + 1],
            )
            .expect("newer schema marker");
        drop(connection);

        let error = SqliteStore::open(&path).expect_err("newer schema must fail");
        assert!(matches!(
            error,
            StorageError::UnsupportedSchema {
                found: 3,
                supported: 2
            }
        ));
    }

    #[test]
    fn version_one_database_upgrades_to_accounts_schema() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("finance.db");
        let connection = Connection::open(&path).expect("connection");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                );
                INSERT INTO schema_migrations (version) VALUES (1);",
            )
            .expect("version one database");
        drop(connection);

        let store = SqliteStore::open(path).expect("upgraded store");

        assert_eq!(store.schema_version().expect("schema version"), 2);
        let table_exists = store
            .connection()
            .expect("connection")
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'accounts')",
                [],
                |row| row.get::<_, bool>(0),
            )
            .expect("accounts table");
        assert!(table_exists);
    }

    #[cfg(unix)]
    #[test]
    fn explicit_database_path_does_not_change_parent_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().expect("temporary directory");
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o755))
            .expect("test permissions");

        SqliteStore::open(directory.path().join("finance.db")).expect("store");

        let mode = fs::metadata(directory.path())
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o755);
    }
}
