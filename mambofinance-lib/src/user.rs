mod category;
mod currency;
mod fund;
mod group;
mod query;
mod transaction;
mod types;

pub use category::CategoryVariant;
pub use category::*;
pub use currency::*;
pub use fund::*;
pub use group::*;
pub use query::*;
pub use transaction::*;
pub(in crate::user) use types::*;

use rusqlite::{Connection, Result};
use std::fs;
use thiserror::Error;
use uuid::Uuid;

use crate::user::InputError::WrongVariant;

/// Maximum character length limit for names.
pub const NAME_LIMIT: usize = 20;
/// Maximum character length limit for descriptions.
pub const DESC_LIMIT: usize = 25;
/// Maximum character length limit for amount strings.
pub const AMOUNT_LIMIT: usize = 10;
/// Maximum character length limit for variant string names.
pub const VARIANT_LIMIT: usize = 8;

/// Represents an active user session tied to a specific SQLite database connection.
#[derive(Debug)]
pub struct User {
    name: String,
    conn: Connection,
}

// region: Error

/// Errors related to user input data validation, constraints, and range checks.
#[derive(Error, Debug)]
pub enum InputError {
    /// Returned when the local storage path directory cannot be created.
    #[error("Failed to create directory: {0}.")]
    InvalidDir(String),

    /// Returned when trying to match or utilize a category with the wrong structure type.
    #[error("{0} already exists but as a different category type.")]
    WrongVariant(String),

    /// Returned when a query references a positional element index out of bounds.
    #[error("No item at index {0}.")]
    InvalidIndex(usize),

    /// Returned when trying to register an entity name that already exists within a domain.
    #[error("{0} already exists in {1}")]
    ExistingItem(String, String),

    /// Returned when trying to edit a category variant that still holds linked active transactions.
    #[error(
        "Category used by {0} transaction(s), restricted from editing variant. Use 'force_edit_variant' to unlink all transactions for this category and edit."
    )]
    CategoryInUse(usize),
}

/// Consolidated top-level error enum handling all operational error variants.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Error, Debug)]
pub enum UserError {
    /// Relayed input validation or structural matching issues.
    #[error(transparent)]
    Input(#[from] InputError),

    /// Relayed date formatting or validation exceptions.
    #[error(transparent)]
    Date(#[from] DateError),

    /// Underlying database connection or statement execution failures.
    #[error(transparent)]
    SQL(#[from] rusqlite::Error),
}

// endregion

// region: New & Check & Get

impl User {
    /// Creates a new `User` instance backed by a persistent file-based SQLite database.
    ///
    /// # Errors
    ///
    /// Returns a `UserError` if the directory cannot be created or the database initialization fails.
    #[allow(dead_code)]
    pub fn new(name: &str) -> Result<Self, UserError> {
        Self::new_at_path(&format!("storage/{}.db", name), name)
    }

    /// Creates a new `User` instance backed by a temporary in-memory SQLite database.
    ///
    /// # Errors
    ///
    /// Returns a `UserError` if the database initialization fails.
    #[allow(dead_code)]
    pub fn new_in_memory(name: &str) -> Result<Self, UserError> {
        Self::new_at_path(":memory:", name)
    }

    // Underlying initializer to construct directories, open connections, and apply schemas.
    fn new_at_path(path: &str, name: &str) -> Result<Self, UserError> {
        if path != ":memory:" {
            fs::create_dir_all("storage")
                .map_err(|e| UserError::Input(InputError::InvalidDir(format!("{}", e))))?;
        }

        let conn = Connection::open(path).map_err(UserError::SQL)?;

        // Enforce cascading foreign keys.
        conn.execute("PRAGMA foreign_keys = ON;", ())
            .map_err(UserError::SQL)?;

        let table_queries = [
            "CREATE TABLE IF NOT EXISTS transactions (
                id BLOB PRIMARY KEY,
                name TEXT,
                description TEXT,
                amount INTEGER,
                currency_id BLOB REFERENCES currencies(id) ON DELETE CASCADE,
                day INTEGER,
                month INTEGER,
                year INTEGER,
                group_id BLOB REFERENCES groups(id) ON DELETE CASCADE,
                category_id BLOB REFERENCES categories(id) ON DELETE CASCADE,
                fund_id BLOB REFERENCES funds(id) ON DELETE CASCADE,
                link_id BLOB REFERENCES transactions(id) ON DELETE CASCADE
            );",
            "CREATE TABLE IF NOT EXISTS groups (
                id BLOB PRIMARY KEY,
                name TEXT
            );",
            "CREATE TABLE IF NOT EXISTS categories (
                id BLOB PRIMARY KEY,
                name TEXT,
                variant INTEGER NOT NULL CHECK (variant in (0,1))
            );",
            "CREATE TABLE IF NOT EXISTS funds (
                id BLOB PRIMARY KEY,
                name TEXT
            );",
            "CREATE TABLE IF NOT EXISTS currencies (
                id BLOB PRIMARY KEY,
                name TEXT
            );",
        ];

        for sql in &table_queries {
            conn.execute(sql, ()).map_err(UserError::SQL)?;
        }

        Ok(User {
            name: String::from(name),
            conn,
        })
    }

    // Confirms whether a registered category variant matches the target enum state.
    fn check_category_variant(&self, id: Uuid, variant: CategoryVariant) -> Result<(), UserError> {
        let existing = self
            .conn
            .query_row(
                "SELECT variant FROM categories WHERE id = ?1",
                [id],
                |row| row.get::<_, CategoryVariant>(0),
            )
            .map_err(UserError::SQL);

        match existing {
            Ok(v) => {
                if v != variant {
                    Err(UserError::Input(WrongVariant(format!("{:?}", v))))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(e),
        }
    }

    // Asserts uniqueness by validating that an entry does not already exist within its mapped table.
    fn check_existing<T: HasLabel>(&self, existing: Option<&str>) -> Result<(), UserError> {
        if let Some(name) = existing {
            return match self.get::<T>(name) {
                Ok(_) => Err(UserError::Input(InputError::ExistingItem(
                    name.to_string(),
                    T::table().to_string(),
                ))),
                Err(UserError::SQL(rusqlite::Error::QueryReturnedNoRows)) => Ok(()),
                Err(e) => Err(e),
            };
        }
        Ok(())
    }

    // Resolves a unique entity UUID based on its name and its relative struct definition table.
    fn get<T: HasLabel>(&self, name: &str) -> Result<Uuid, UserError> {
        let query = format!("SELECT id FROM {} WHERE name = ?1", T::table());
        self.conn
            .query_row(&query, [Label::fmt(name)], |row| row.get(0))
            .map_err(UserError::SQL)
    }
}

// endregion

// region: Add

impl User {
    /// Adds a standalone single-entry transaction to the database ledger.
    ///
    /// # Errors
    ///
    /// Returns `UserError` if relational elements are missing, arguments fail limits, or query execution fails.
    #[allow(clippy::too_many_arguments)]
    pub fn add_transaction(
        &self,
        name: &str,
        description: Option<&str>,
        (amount, currency): (i64, &str),
        (day, month, year): (u8, u8, u16),
        group: &str,
        category: &str,
        fund: &str,
    ) -> Result<&Self, UserError> {
        let category = self.get::<Category>(category)?;
        self.check_category_variant(category, CategoryVariant::Single)?;

        let group = self.get::<Group>(group)?;
        let fund = self.get::<Fund>(fund)?;
        let currency = self.get::<Currency>(currency)?;
        let amount = RawAmount::new(amount);
        let date = Date::new(day, month, year)?;
        let label = Label::new(name, description);

        self.add::<Transaction>(&label, None, |conn: &Connection, label| {
            conn.execute(
                "INSERT INTO transactions (id, name, description, amount, currency_id, day, month, year, group_id, fund_id, category_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    label.id, label.name, label.description, amount.value, currency,
                    date.day, date.month, date.year, group, fund, category,
                ],
            )
        })
    }

    /// Adds a double-entry paired transaction (e.g., system transfers) wrapped in an atomic database transaction block.
    ///
    /// # Errors
    ///
    /// Returns `UserError` if constraints are violated, variants clash, or database execution fails.
    #[allow(clippy::too_many_arguments)]
    pub fn add_paired_transaction(
        &self,
        name: &str,
        description: Option<&str>,
        (source_amount, source_currency): (i64, &str),
        (target_amount, target_currency): (i64, &str),
        (day, month, year): (u8, u8, u16),
        group: &str,
        category: &str,
        source_fund: &str,
        target_fund: &str,
    ) -> Result<&Self, UserError> {
        let category = self.get::<Category>(category)?;
        self.check_category_variant(category, CategoryVariant::Paired)?;

        let group = self.get::<Group>(group)?;
        let date = Date::new(day, month, year)?;

        let source_label = Label::new(name, description);
        let source_amount = RawAmount::new(source_amount);
        let source_currency = self.get::<Currency>(source_currency)?;
        let source_fund = self.get::<Fund>(source_fund)?;
        let target_label = Label::new(name, description);
        let target_amount = RawAmount::new(target_amount);
        let target_currency = self.get::<Currency>(target_currency)?;
        let target_fund = self.get::<Fund>(target_fund)?;

        let tx = self.conn.unchecked_transaction().map_err(UserError::SQL)?;

        // Temporarily defer constraints to ensure mutually referencing linked transactions pass validation.
        tx.execute("PRAGMA defer_foreign_keys = ON;", ())?;

        // Process the outgoing side of the entry.
        self.add::<Transaction>(&source_label, None, |conn, label| {
            conn.execute(
                "INSERT INTO transactions (id, name, description, amount, currency_id, day, month, year, group_id, fund_id, category_id, link_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    label.id, label.name, label.description, source_amount.value, source_currency,
                    date.day, date.month, date.year, group, source_fund, category, target_label.id
                ],
            )
        })?;

        // Process the incoming side of the entry.
        self.add::<Transaction>(&target_label, None, |conn, label| {
            conn.execute(
                "INSERT INTO transactions (id, name, description, amount, currency_id, day, month, year, group_id, fund_id, category_id, link_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    label.id, label.name, label.description, target_amount.value, target_currency,
                    date.day, date.month, date.year, group, target_fund, category, source_label.id
                ],
            )
        })?;

        tx.commit()?;
        Ok(self)
    }

    /// Registers a new tracking transaction group if the identifier is unique.
    pub fn add_group(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add::<Group>(&label, Some(name), |conn, label| {
            conn.execute(
                "INSERT INTO groups (id, name) VALUES (?1, ?2)",
                rusqlite::params![label.id, label.name],
            )
        })
    }

    /// Registers a new single-entry type category if the identifier is unique.
    pub fn add_category(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add::<Category>(&label, Some(name), |conn, label| {
            conn.execute(
                "INSERT INTO categories (id, name, variant) VALUES (?1, ?2, ?3)",
                rusqlite::params![label.id, label.name, 0],
            )
        })
    }

    /// Registers a new double-entry paired category if the identifier is unique.
    pub fn add_paired_category(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add::<Category>(&label, Some(name), |conn, label| {
            conn.execute(
                "INSERT INTO categories (id, name, variant) VALUES (?1, ?2, ?3)",
                rusqlite::params![label.id, label.name, 1],
            )
        })
    }

    /// Registers a new asset fund or balance account if the identifier is unique.
    pub fn add_fund(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add::<Fund>(&label, Some(name), |conn, label| {
            conn.execute(
                "INSERT INTO funds (id, name) VALUES (?1, ?2)",
                rusqlite::params![label.id, label.name],
            )
        })
    }

    /// Registers a new tracking currency variant if the identifier is unique.
    pub fn add_currency(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add::<Currency>(&label, Some(name), |conn, label| {
            conn.execute(
                "INSERT INTO currencies (id, name) VALUES (?1, ?2)",
                rusqlite::params![label.id, label.name],
            )
        })
    }

    // Helper wrapper ensuring uniqueness constraints are checked ahead of structural insertion.
    fn add<T: HasLabel>(
        &self,
        label: &Label,
        existing: Option<&str>,
        insert: impl FnOnce(&rusqlite::Connection, &Label) -> rusqlite::Result<usize>,
    ) -> Result<&Self, UserError> {
        self.check_existing::<T>(existing)?;
        insert(&self.conn, label)
            .map(|_| ())
            .map_err(UserError::SQL)?;
        Ok(self)
    }
}

// endregion

// region: ls & Query

impl User {
    // Compiles full transactional datasets combined across related tables via relational joins.
    fn ls_transaction(&self) -> Result<Vec<Transaction>, UserError> {
        self.ls_table(
            "SELECT
                t.id, t.name, t.description,
                t.amount,
                cur.id, cur.name,
                (SELECT COUNT(*) FROM transactions sub WHERE sub.currency_id = cur.id) as currency_count,
                t.day, t.month, t.year,
                g.id, g.name,
                (SELECT COUNT(*) FROM transactions sub WHERE sub.group_id = g.id) as group_count,
                cat.id, cat.name, cat.variant,
                (SELECT COUNT(*) FROM transactions sub WHERE sub.category_id = cat.id) as category_count,
                f.id, f.name,
                (SELECT COUNT(*) FROM transactions sub WHERE sub.fund_id = f.id) as fund_count,
                t.link_id
            FROM transactions t
            JOIN currencies cur ON t.currency_id = cur.id
            JOIN groups g       ON t.group_id    = g.id
            JOIN categories cat ON t.category_id = cat.id
            JOIN funds f        ON t.fund_id     = f.id",
            Transaction::from_row,
        )
    }

    // Fetches registered group aggregates along with active transaction tracking metrics.
    fn ls_group(&self) -> Result<Vec<Group>, UserError> {
        self.ls_table(
            "SELECT 
                i.id, i.name, 
                COUNT(t.id) as transaction_count 
            FROM groups i 
            LEFT JOIN transactions t ON i.id = t.group_id
            GROUP BY i.id, i.name",
            Group::from_row,
        )
    }

    // Fetches registered category structural options along with relational transaction use metrics.
    fn ls_category(&self) -> Result<Vec<Category>, UserError> {
        self.ls_table(
            "SELECT 
                i.id, i.name, i.variant,
                COUNT(t.id) as transaction_count 
            FROM categories i 
            LEFT JOIN transactions t ON i.id = t.category_id
            GROUP BY i.id, i.name, i.variant",
            Category::from_row,
        )
    }

    // Fetches registered ledger accounts/funds alongside their current total transaction usage metrics.
    fn ls_fund(&self) -> Result<Vec<Fund>, UserError> {
        self.ls_table(
            "SELECT 
                i.id, i.name, 
                COUNT(t.id) as transaction_count 
            FROM funds i 
            LEFT JOIN transactions t ON i.id = t.fund_id
            GROUP BY i.id, i.name",
            Fund::from_row,
        )
    }

    // Fetches tracked currencies alongside their transaction execution usage counts.
    fn ls_currency(&self) -> Result<Vec<Currency>, UserError> {
        self.ls_table(
            "SELECT 
                i.id, i.name, 
                COUNT(t.id) as transaction_count 
            FROM currencies i 
            LEFT JOIN transactions t ON i.id = t.currency_id
            GROUP BY i.id, i.name",
            Currency::from_row,
        )
    }

    // Prepares raw string queries and maps resulting database records to vector targets.
    fn ls_table<T: HasLabel>(
        &self,
        query: &str,
        from_row: impl Fn(&rusqlite::Row) -> rusqlite::Result<T>,
    ) -> Result<Vec<T>, UserError> {
        let mut stmt = self.conn.prepare(query).map_err(UserError::SQL)?;
        let rows = stmt.query_map([], from_row).map_err(UserError::SQL)?;
        rows.collect::<rusqlite::Result<Vec<T>>>()
            .map_err(UserError::SQL)
    }
}

impl User {
    /// Constructs a dataset query wrapper focused around all ledger transactions.
    pub fn transactions(&self) -> Result<Query<'_, Transaction>, UserError> {
        Ok(Query::new(
            self,
            self.ls_transaction()?,
            "TRANSACTION",
            vec![
                ("NO", 0, FieldVariant::Index),
                ("NAME", NAME_LIMIT, FieldVariant::Limit),
                ("DESCRIPTION", DESC_LIMIT, FieldVariant::Limit),
                ("AMOUNT", AMOUNT_LIMIT + 13, FieldVariant::Limit),
                ("DATE", 11, FieldVariant::Static),
                ("GROUP", NAME_LIMIT, FieldVariant::Limit),
                ("CATEGORY", NAME_LIMIT, FieldVariant::Limit),
                ("FUND", NAME_LIMIT, FieldVariant::Limit),
                ("LINK", 0, FieldVariant::Link),
            ],
        ))
    }

    /// Constructs a dataset query wrapper focused around registered category tracking groups.
    pub fn groups(&self) -> Result<Query<'_, Group>, UserError> {
        Ok(Query::new(
            self,
            self.ls_group()?,
            "GROUP",
            vec![
                ("NO", 0, FieldVariant::Index),
                ("NAME", NAME_LIMIT, FieldVariant::Limit),
                ("COUNT", 0, FieldVariant::Count),
            ],
        ))
    }

    /// Constructs a dataset query wrapper focused around registered accounting categories.
    pub fn categories(&self) -> Result<Query<'_, Category>, UserError> {
        Ok(Query::new(
            self,
            self.ls_category()?,
            "CATEGORY",
            vec![
                ("NO", 0, FieldVariant::Index),
                ("NAME", NAME_LIMIT, FieldVariant::Limit),
                ("TYPE", VARIANT_LIMIT, FieldVariant::Static),
                ("COUNT", 0, FieldVariant::Count),
            ],
        ))
    }

    /// Constructs a dataset query wrapper focused around active balance funds and accounts.
    pub fn funds(&self) -> Result<Query<'_, Fund>, UserError> {
        Ok(Query::new(
            self,
            self.ls_fund()?,
            "FUND",
            vec![
                ("NO", 0, FieldVariant::Index),
                ("NAME", NAME_LIMIT, FieldVariant::Limit),
                ("COUNT", 0, FieldVariant::Count),
            ],
        ))
    }

    /// Constructs a dataset query wrapper focused around registered global currencies.
    pub fn currencies(&self) -> Result<Query<'_, Currency>, UserError> {
        Ok(Query::new(
            self,
            self.ls_currency()?,
            "CURRENCY",
            vec![
                ("NO", 0, FieldVariant::Index),
                ("NAME", NAME_LIMIT, FieldVariant::Limit),
                ("COUNT", 0, FieldVariant::Count),
            ],
        ))
    }
}

// endregion

// region: Test

#[cfg(test)]
mod tests {
    use super::*;

    // region: User::new_at_path (private)

    /// Verifies initialization parameters produce a valid user file profile context containing matching identity metrics.
    #[test]
    fn new_at_path_creates_an_in_memory_user_with_given_name() {
        // Arrange & Act
        let user = User::new_at_path(":memory:", "alice");

        // Assert
        assert!(user.is_ok());
        assert_eq!(user.unwrap().name, "alice");
    }

    /// Verifies core configuration maps populate the full scope of internal structural storage schema components.
    #[test]
    fn new_at_path_creates_all_expected_tables() {
        // Arrange
        let user = User::new_at_path(":memory:", "alice").expect("user creation should succeed");

        // Act
        let table_count: i64 = user
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN
                 ('transactions','groups','categories','funds','currencies')",
                (),
                |row| row.get(0),
            )
            .expect("table count query should succeed");

        // Assert
        assert_eq!(table_count, 5);
    }

    /// Verifies that operational database engines run with active relational dependency enforcement routines enabled.
    #[test]
    fn new_at_path_enables_foreign_key_enforcement() {
        // Arrange
        let user = User::new_at_path(":memory:", "alice").expect("user creation should succeed");

        // Act
        let fk_enabled: i64 = user
            .conn
            .query_row("PRAGMA foreign_keys;", (), |row| row.get(0))
            .expect("pragma query should succeed");

        // Assert
        assert_eq!(fk_enabled, 1);
    }

    // endregion

    // region: User::check_category_variant (private)

    /// Verifies target categorization structural matching filters authorize exact matching definitions.
    #[test]
    fn check_category_variant_succeeds_when_variant_matches() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_category("Food").expect("add_category failed");
        let id = user
            .get::<Category>("Food")
            .expect("category lookup failed");

        // Act
        let result = user.check_category_variant(id, CategoryVariant::Single);

        // Assert
        assert!(result.is_ok());
    }

    /// Verifies asset entry parsing structures block incompatible structural properties across validation states.
    #[test]
    fn check_category_variant_fails_when_variant_mismatches() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_category("Food").expect("add_category failed");
        let id = user
            .get::<Category>("Food")
            .expect("category lookup failed");

        // Act
        let result = user.check_category_variant(id, CategoryVariant::Paired);

        // Assert
        assert!(matches!(result, Err(UserError::Input(WrongVariant(_)))));
    }

    /// Verifies checking routines surface missing record keys as lower level interface failures.
    #[test]
    fn check_category_variant_propagates_sql_error_for_unknown_id() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        let bogus_id = Uuid::new_v4();

        // Act
        let result = user.check_category_variant(bogus_id, CategoryVariant::Single);

        // Assert
        assert!(matches!(result, Err(UserError::SQL(_))));
    }

    // endregion

    // region: User::check_existing (private)

    /// Verifies uniqueness checking blocks gracefully bypass empty identity evaluation workflows.
    #[test]
    fn check_existing_succeeds_when_name_is_none() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.check_existing::<Group>(None);

        // Assert
        assert!(result.is_ok());
    }

    /// Verifies allocation verification confirms clearance marks for unassigned structural tokens.
    #[test]
    fn check_existing_succeeds_when_name_does_not_exist_yet() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.check_existing::<Group>(Some("Personal"));

        // Assert
        assert!(result.is_ok());
    }

    /// Verifies registration guards trap overlapping names inside matching functional targets.
    #[test]
    fn check_existing_fails_when_name_already_exists() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");

        // Act
        let result = user.check_existing::<Group>(Some("Personal"));

        // Assert
        assert!(matches!(
            result,
            Err(UserError::Input(InputError::ExistingItem(_, _)))
        ));
    }

    // endregion

    // region: User::get (private)

    /// Verifies index mapping strategies parse record markers correctly from primary target labels.
    #[test]
    fn get_resolves_the_id_of_an_existing_entity() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");

        // Act
        let result = user.get::<Group>("Personal");

        // Assert
        assert!(result.is_ok());
    }

    /// Verifies structural text filters match targets independently of raw user input formatting styles.
    #[test]
    fn get_is_case_and_format_insensitive_via_label_fmt() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal Finance")
            .expect("add_group failed");

        // Act
        let result = user.get::<Group>("personal_finance");

        // Assert
        assert!(result.is_ok());
    }

    /// Verifies engine routing reports unmapped query tracking parameters cleanly as targeted empty states.
    #[test]
    fn get_returns_sql_error_for_a_nonexistent_entity() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.get::<Group>("Nonexistent");

        // Assert
        assert!(matches!(
            result,
            Err(UserError::SQL(rusqlite::Error::QueryReturnedNoRows))
        ));
    }

    // endregion

    // region: User::add (private)

    /// Verifies that foundational registration logic inserts valid entities into targeted tracking vectors.
    #[test]
    fn add_inserts_a_new_entity_when_unique() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.add_group("Personal");

        // Assert
        assert!(result.is_ok());
        assert!(user.get::<Group>("Personal").is_ok());
    }

    /// Verifies that core record wrappers enforce uniform name uniqueness boundaries inside individual tables.
    #[test]
    fn add_rejects_a_duplicate_entity_name() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("first add_group failed");

        // Act
        let result = user.add_group("Personal");

        // Assert
        assert!(matches!(
            result,
            Err(UserError::Input(InputError::ExistingItem(_, _)))
        ));
    }

    // endregion

    // region: User::ls_transaction (private)

    /// Verifies compilation routines return empty arrays cleanly ahead of entry registrations.
    #[test]
    fn ls_transaction_returns_an_empty_vec_when_no_transactions_exist() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.ls_transaction();

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Verifies operational extraction loops collect multi-table record joins safely into structured entities.
    #[test]
    fn ls_transaction_returns_inserted_transactions() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_category("Food").expect("add_category failed");
        user.add_transaction(
            "Groceries",
            None,
            (1050, "USD"),
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        )
        .expect("add_transaction failed");

        // Act
        let result = user.ls_transaction();

        // Assert
        assert!(result.is_ok());
        let rows = result.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label.name, "Groceries");
    }

    // endregion

    // region: User::ls_group / ls_category / ls_fund / ls_currency (private)

    /// Verifies aggregate group compilers output baseline tracking assignments properly before active operational attachments.
    #[test]
    fn ls_group_returns_zero_count_for_an_unused_group() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");

        // Act
        let result = user.ls_group();

        // Assert
        let rows = result.expect("ls_group should succeed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].count, 0);
    }

    /// Verifies system calculation layers accurately increment active structural usages upon resource generation.
    #[test]
    fn ls_category_returns_correct_count_after_transactions_are_added() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_category("Food").expect("add_category failed");
        user.add_transaction(
            "A",
            None,
            (100, "USD"),
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        )
        .expect("add_transaction failed");

        // Act
        let result = user.ls_category();

        // Assert
        let rows = result.expect("ls_category should succeed");
        let food = rows
            .iter()
            .find(|c| c.label.name == "Food")
            .expect("Food should exist");
        assert_eq!(food.count, 1);
    }

    /// Verifies target account compiler summaries output pristine reference limits prior to active system entries.
    #[test]
    fn ls_fund_returns_zero_count_for_an_unused_fund() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_fund("Cash").expect("add_fund failed");

        // Act
        let result = user.ls_fund();

        // Assert
        let rows = result.expect("ls_fund should succeed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].count, 0);
    }

    /// Verifies currency evaluation frameworks reflect neutral allocation benchmarks during early staging setup.
    #[test]
    fn ls_currency_returns_zero_count_for_an_unused_currency() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_currency("USD").expect("add_currency failed");

        // Act
        let result = user.ls_currency();

        // Assert
        let rows = result.expect("ls_currency should succeed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].count, 0);
    }

    // endregion

    // region: User::ls_table (private)

    /// Verifies low-level collection loops parse syntax problems upstream into clear transactional exceptions.
    #[test]
    fn ls_table_propagates_a_malformed_query_as_a_sql_error() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.ls_table("SELECT * FROM nonexistent_table", Group::from_row);

        // Assert
        assert!(matches!(result, Err(UserError::SQL(_))));
    }

    // endregion

    // region: User::new

    /// Verifies volatile allocation wrappers construct safe runtime operational configurations cleanly.
    #[test]
    fn new_in_memory_creates_a_usable_user() {
        // Arrange & Act
        let result = User::new_in_memory("bob");

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "bob");
    }

    // endregion

    // region: User::add_transaction

    /// Verifies standard journal entry processing pipelines map data correctly into corresponding storage properties.
    #[test]
    fn add_transaction_inserts_a_single_entry_transaction() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_category("Food").expect("add_category failed");

        // Act
        let result = user.add_transaction(
            "Groceries",
            Some("Weekly shop"),
            (1050, "USD"),
            (15, 6, 2026),
            "Personal",
            "Food",
            "Cash",
        );

        // Assert
        assert!(result.is_ok());
        let rows = user
            .ls_transaction()
            .expect("ls_transaction should succeed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label.description, Some(String::from("Weekly shop")));
    }

    /// Verifies single entries intercept and reject values paired against complex coordinate rules.
    #[test]
    fn add_transaction_rejects_a_paired_only_category() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_paired_category("Transfer")
            .expect("add_paired_category failed");

        // Act
        let result = user.add_transaction(
            "Move",
            None,
            (100, "USD"),
            (1, 1, 2026),
            "Personal",
            "Transfer",
            "Cash",
        );

        // Assert
        assert!(matches!(result, Err(UserError::Input(WrongVariant(_)))));
    }

    /// Verifies calendar processing bounds drop incorrect or structurally impossible time intervals ahead of record storage.
    #[test]
    fn add_transaction_rejects_an_invalid_date() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_category("Food").expect("add_category failed");

        // Act
        let result = user.add_transaction(
            "Groceries",
            None,
            (100, "USD"),
            (31, 4, 2026), // April has 30 days
            "Personal",
            "Food",
            "Cash",
        );

        // Assert
        assert!(matches!(result, Err(UserError::Date(_))));
    }

    /// Verifies entry processors fail safely when pointing to missing relational accounts or properties.
    #[test]
    fn add_transaction_rejects_an_unknown_fund() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_category("Food").expect("add_category failed");

        // Act
        let result = user.add_transaction(
            "Groceries",
            None,
            (100, "USD"),
            (1, 1, 2026),
            "Personal",
            "Food",
            "Nonexistent Fund",
        );

        // Assert
        assert!(matches!(
            result,
            Err(UserError::SQL(rusqlite::Error::QueryReturnedNoRows))
        ));
    }

    // endregion

    // region: User::add_paired_transaction

    /// Verifies complex multi-leg journal transfers map balanced structural dependencies cleanly onto both nodes.
    #[test]
    fn add_paired_transaction_inserts_both_linked_legs() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_fund("Bank").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_paired_category("Transfer")
            .expect("add_paired_category failed");

        // Act
        let result = user.add_paired_transaction(
            "Move",
            None,
            (500, "USD"),
            (500, "USD"),
            (1, 1, 2026),
            "Personal",
            "Transfer",
            "Cash",
            "Bank",
        );

        // Assert
        assert!(result.is_ok());
        let rows = user
            .ls_transaction()
            .expect("ls_transaction should succeed");
        assert_eq!(rows.len(), 2);
        assert!(rows[0].link.is_some());
        assert!(rows[1].link.is_some());
        assert_eq!(rows[0].link, Some(rows[1].label.id));
        assert_eq!(rows[1].link, Some(rows[0].label.id));
    }

    /// Verifies double-entry operations intercept and reject assignments targeting flat structural categories.
    #[test]
    fn add_paired_transaction_rejects_a_single_only_category() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_fund("Bank").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_category("Food").expect("add_category failed");

        // Act
        let result = user.add_paired_transaction(
            "Move",
            None,
            (500, "USD"),
            (500, "USD"),
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
            "Bank",
        );

        // Assert
        assert!(matches!(result, Err(UserError::Input(WrongVariant(_)))));
    }

    /// Verifies atomic isolation strategies discard whole compound tracking states if single elements cross error boundaries.
    #[test]
    fn add_paired_transaction_rolls_back_on_failure_inserting_neither_leg() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_paired_category("Transfer")
            .expect("add_paired_category failed");

        // Act
        let result = user.add_paired_transaction(
            "Move",
            None,
            (500, "USD"),
            (500, "USD"),
            (1, 1, 2026),
            "Personal",
            "Transfer",
            "Cash",
            "Nonexistent",
        );

        // Assert
        assert!(result.is_err());
        let rows = user
            .ls_transaction()
            .expect("ls_transaction should succeed");
        assert!(rows.is_empty());
    }

    // endregion

    // region: User::add_group / add_category / add_paired_category / add_fund / add_currency

    /// Verifies division registers assign unique tracking boundaries correctly into isolated targets.
    #[test]
    fn add_group_registers_a_new_group() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.add_group("Personal");

        // Assert
        assert!(result.is_ok());
        assert_eq!(user.ls_group().unwrap().len(), 1);
    }

    /// Verifies classification tools register base categorical divisions accurately under single-leg markers.
    #[test]
    fn add_category_registers_a_single_variant_category() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.add_category("Food");

        // Assert
        assert!(result.is_ok());
        let rows = user.ls_category().unwrap();
        assert_eq!(rows[0].variant, CategoryVariant::Single);
    }

    /// Verifies classification engines construct paired category tracking properties cleanly with proper variants.
    #[test]
    fn add_paired_category_registers_a_paired_variant_category() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.add_paired_category("Transfer");

        // Assert
        assert!(result.is_ok());
        let rows = user.ls_category().unwrap();
        assert_eq!(rows[0].variant, CategoryVariant::Paired);
    }

    /// Verifies asset manager nodes establish localized account storage maps accurately upon command execution.
    #[test]
    fn add_fund_registers_a_new_fund() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.add_fund("Cash");

        // Assert
        assert!(result.is_ok());
        assert_eq!(user.ls_fund().unwrap().len(), 1);
    }

    /// Verifies currency tracker frames map individual identifier values successfully into active records.
    #[test]
    fn add_currency_registers_a_new_currency() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let result = user.add_currency("USD");

        // Assert
        assert!(result.is_ok());
        assert_eq!(user.ls_currency().unwrap().len(), 1);
    }

    // endregion

    // region: User::transactions / groups / categories / funds / currencies (Query builders)

    /// Verifies data queries structure clear headers matching expected presentation requirements.
    #[test]
    fn transactions_build_a_query_with_expected_headers() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");

        // Act
        let query = user.transactions();

        // Assert
        assert!(query.is_ok());
        let headers: Vec<String> = query
            .unwrap()
            .headers
            .into_iter()
            .map(|(h, ..)| h)
            .collect();
        assert_eq!(
            headers,
            vec![
                "NO",
                "NAME",
                "DESCRIPTION",
                "AMOUNT",
                "DATE",
                "GROUP",
                "CATEGORY",
                "FUND",
                "LINK"
            ]
        );
    }

    /// Verifies presentation engines configure header metadata tags correctly based on the entity domain query targets.
    #[test]
    fn groups_builds_a_query_with_expected_title() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_group("Personal").expect("add_group failed");

        // Act
        let query = user.groups();

        // Assert
        assert!(query.is_ok());
        let query = query.unwrap();
        assert_eq!(query.title, "GROUP");
        assert_eq!(query.rows.len(), 1);
    }

    /// Verifies categorization filters extract all active entries into view targets.
    #[test]
    fn categories_builds_a_query_containing_registered_categories() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_category("Food").expect("add_category failed");

        // Act
        let query = user.categories();

        // Assert
        assert!(query.is_ok());
        assert_eq!(query.unwrap().rows.len(), 1);
    }

    /// Verifies resource tracking metrics route stored entities accurately to analytical view grids.
    #[test]
    fn funds_builds_a_query_containing_registered_funds() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_fund("Cash").expect("add_fund failed");

        // Act
        let query = user.funds();

        // Assert
        assert!(query.is_ok());
        assert_eq!(query.unwrap().rows.len(), 1);
    }

    /// Verifies standard system query layers forward assigned currency components correctly without translation loss.
    #[test]
    fn currencies_builds_a_query_containing_registered_currencies() {
        // Arrange
        let user = User::new_in_memory("alice").expect("user creation should succeed");
        user.add_currency("USD").expect("add_currency failed");

        // Act
        let query = user.currencies();

        // Assert
        assert!(query.is_ok());
        assert_eq!(query.unwrap().rows.len(), 1);
    }

    // endregion
}

// endregion
