// Modules
mod category;
mod currency;
mod fund;
mod group;
mod query;
mod transaction;
mod types;

// Re-exports/Imports from internal modules
pub use category::CategoryVariant;
use category::*;
use currency::*;
use fund::*;
use group::*;
use query::*;
use transaction::*;
use types::*;

// External dependencies
use rusqlite::{Connection, Result};
use std::fmt::Display;
use std::fs;
use thiserror::Error;
use uuid::Uuid;

use crate::user::InputError::WrongVariant;

/// Maximum character length limits for string inputs.
pub const NAME_LIMIT: usize = 20;
pub const DESC_LIMIT: usize = 25;
pub const AMOUNT_LIMIT: usize = 10;
pub const VARIANT_LIMIT: usize = 8;

/// Represents a user session tied to a specific SQLite database connection.
#[allow(dead_code)]
pub struct User {
    label: Label,
    conn: Connection,
}

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

    /// Internal helper to initialize a connection and create the schema at the specified path.
    fn new_at_path(path: &str, name: &str) -> Result<Self, UserError> {
        if path != ":memory:" {
            fs::create_dir_all("storage")
                .map_err(|e| UserError::Input(InputError::InvalidDir(format!("{}", e))))?;
        }

        let conn = Connection::open(path).map_err(UserError::SQL)?;

        // Enforce foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON;", ())
            .map_err(UserError::SQL)?;

        // Base schema definition for the accounting engine
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
                link_id BLOB REFERENCES transactions(id) ON DELETE SET NULL
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
            label: Label::new(name, Some("USER")),
            conn,
        })
    }
}

impl User {
    /// Adds a standalone single-entry transaction to the database.
    ///
    /// # Errors
    ///
    /// Returns `UserError` if any relation is not found, parameters are invalid, or SQL execution fails.
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
        let category = self.get_category(category)?;
        self.check_category_variant(category, CategoryVariant::Single)?;

        let group = self.get_group(group)?;
        let fund = self.get_fund(fund)?;
        let currency = self.get_currency(currency)?;

        let amount = RawAmount::new(amount);
        let date = Date::new(day, month, year)?;
        let label = Label::new(name, description);

        self.add_to_table(&label, |conn: &Connection, label| {
            conn.execute(
                "INSERT INTO transactions (id, name, description, amount, currency_id, day, month, year, group_id, fund_id, category_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    label.id, label.name, label.description, amount.value, currency,
                    date.day, date.month, date.year, group, fund, category,
                ],
            )
            .map(|_| ())
        })?;

        Ok(self)
    }

    /// Adds a double-entry paired transaction (e.g., transfers, currency exchanges) wrapped in a SQL transaction.
    ///
    /// # Errors
    ///
    /// Returns `UserError` if verification fails or the database transaction fails to execute or commit.
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
        let category = self.get_category(category)?;
        self.check_category_variant(category, CategoryVariant::Paired)?;

        let group = self.get_group(group)?;
        let date = Date::new(day, month, year)?;

        let source_currency = self.get_currency(source_currency)?;
        let source_amount = RawAmount::new(source_amount);
        let target_currency = self.get_currency(target_currency)?;
        let target_amount = RawAmount::new(target_amount);
        let source_fund = self.get_fund(source_fund)?;
        let target_fund = self.get_fund(target_fund)?;
        let source_label = Label::new(name, description);
        let target_label = Label::new(name, description);

        let tx = self.conn.unchecked_transaction().map_err(UserError::SQL)?;

        // Defer constraints to allow mutually referencing transactions to be inserted
        tx.execute("PRAGMA defer_foreign_keys = ON;", ())?;

        // Insert source ledger entry
        self.add_to_table(&source_label, |conn, label| {
            conn.execute(
                "INSERT INTO transactions (id, name, description, amount, currency_id, day, month, year, group_id, fund_id, category_id, link_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    label.id, label.name, label.description, source_amount.value, source_currency,
                    date.day, date.month, date.year, group, source_fund, category, target_label.id
                ],
            )
            .map(|_| ())
        })?;

        // Insert target ledger entry
        self.add_to_table(&target_label, |conn, label| {
            conn.execute(
                "INSERT INTO transactions (id, name, description, amount, currency_id, day, month, year, group_id, fund_id, category_id, link_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    label.id, label.name, label.description, target_amount.value, target_currency,
                    date.day, date.month, date.year, group, target_fund, category, source_label.id
                ],
            )
            .map(|_| ())
        })?;

        tx.commit()?;

        Ok(self)
    }

    /// Registers a new transaction group if it does not already exist.
    pub fn add_group(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table::<Group>(name, &label, |conn, label| {
            conn.execute(
                "INSERT INTO groups (id, name) VALUES (?1, ?2)",
                rusqlite::params![label.id, label.name],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }

    /// Registers a new single-variant category if it does not already exist.
    pub fn add_category(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table::<Category>(name, &label, |conn, label| {
            conn.execute(
                "INSERT INTO categories (id, name, variant) VALUES (?1, ?2, ?3)",
                rusqlite::params![label.id, label.name, 0],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }

    /// Registers a new paired-variant category if it does not already exist.
    pub fn add_paired_category(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table::<Category>(name, &label, |conn, label| {
            conn.execute(
                "INSERT INTO categories (id, name, variant) VALUES (?1, ?2, ?3)",
                rusqlite::params![label.id, label.name, 1],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }

    /// Registers a new asset fund/account if it does not already exist.
    pub fn add_fund(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table::<Fund>(name, &label, |conn, label| {
            conn.execute(
                "INSERT INTO funds (id, name) VALUES (?1, ?2)",
                rusqlite::params![label.id, label.name],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }

    /// Registers a new fiat or cryptocurrency asset if it does not already exist.
    pub fn add_currency(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table::<Currency>(name, &label, |conn, label| {
            conn.execute(
                "INSERT INTO currencies (id, name) VALUES (?1, ?2)",
                rusqlite::params![label.id, label.name],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }

    /// Helper variant that inserts a record only if the unique name constraint is satisfied.
    fn add_unique_to_table<T: HasLabel>(
        &self,
        name: &str,
        label: &Label,
        insert: impl FnOnce(&rusqlite::Connection, &Label) -> rusqlite::Result<()>,
    ) -> Result<(), UserError> {
        let existing = self.get_from_table::<T>(name);
        match existing {
            Ok(_) => Ok(()),
            Err(UserError::SQL(rusqlite::Error::QueryReturnedNoRows)) => {
                insert(&self.conn, label).map_err(UserError::SQL)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Standard closure wrapper executing insertions on the underlying SQLite connection.
    fn add_to_table(
        &self,
        label: &Label,
        insert: impl FnOnce(&rusqlite::Connection, &Label) -> rusqlite::Result<()>,
    ) -> Result<(), UserError> {
        insert(&self.conn, label).map_err(UserError::SQL)
    }
}

impl User {
    /// Asserts that a category matches the expected structural variant (Single vs Paired).
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

    fn get_group(&self, name: &str) -> Result<Uuid, UserError> {
        self.get_from_table::<Group>(name)
    }

    fn get_category(&self, name: &str) -> Result<Uuid, UserError> {
        self.get_from_table::<Category>(name)
    }

    fn get_fund(&self, name: &str) -> Result<Uuid, UserError> {
        self.get_from_table::<Fund>(name)
    }

    fn get_currency(&self, name: &str) -> Result<Uuid, UserError> {
        self.get_from_table::<Currency>(name)
    }

    /// Extracts a unique entity UUID based on its string representation and table descriptor.
    fn get_from_table<T: HasLabel>(&self, name: &str) -> Result<Uuid, UserError> {
        let query = format!("SELECT id FROM {} WHERE name = ?1", T::table());
        self.conn
            .query_row(&query, [Label::fmt(name)], |row| row.get(0))
            .map_err(UserError::SQL)
    }
}

impl User {
    /// Compiles all elements from the `transactions` table with relational joins.
    fn ls_transaction(&self) -> Result<Vec<Transaction>, UserError> {
        self.ls_query(
            "SELECT
                t.id, t.name, t.description,
                t.amount, cur.id, cur.name,
                t.day, t.month, t.year,
                g.id, g.name,
                cat.id, cat.name, cat.variant,
                f.id, f.name,
                t.link_id
            FROM transactions t
            JOIN currencies cur ON t.currency_id = cur.id
            JOIN groups g       ON t.group_id    = g.id
            JOIN categories cat ON t.category_id = cat.id
            JOIN funds f        ON t.fund_id      = f.id",
            Transaction::from_row,
        )
    }

    fn ls_group(&self) -> Result<Vec<Group>, UserError> {
        self.ls_table(Group::from_row)
    }

    fn ls_category(&self) -> Result<Vec<Category>, UserError> {
        self.ls_table(Category::from_row)
    }

    fn ls_fund(&self) -> Result<Vec<Fund>, UserError> {
        self.ls_table(Fund::from_row)
    }

    fn ls_currency(&self) -> Result<Vec<Currency>, UserError> {
        self.ls_table(Currency::from_row)
    }

    /// Maps whole table records to specific data models implementing `HasLabel`.
    fn ls_table<T: HasLabel>(
        &self,
        from_row: impl Fn(&rusqlite::Row) -> rusqlite::Result<T>,
    ) -> Result<Vec<T>, UserError> {
        let query = format!("SELECT * FROM {}", T::table());
        let mut stmt = self.conn.prepare(&query).map_err(UserError::SQL)?;
        let rows = stmt.query_map([], from_row).map_err(UserError::SQL)?;
        rows.collect::<rusqlite::Result<Vec<T>>>()
            .map_err(UserError::SQL)
    }

    /// Queries records using custom parameterized/joined SQL strings.
    fn ls_query<T>(
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
    pub fn transactions(&self) -> Result<Query<'_, Transaction>, UserError> {
        Ok(Query {
            user: self,
            rows: self.ls_transaction()?,
        })
    }
    pub fn groups(&self) -> Result<Query<'_, Group>, UserError> {
        Ok(Query {
            user: self,
            rows: self.ls_group()?,
        })
    }
    pub fn categories(&self) -> Result<Query<'_, Category>, UserError> {
        Ok(Query {
            user: self,
            rows: self.ls_category()?,
        })
    }
    pub fn funds(&self) -> Result<Query<'_, Fund>, UserError> {
        Ok(Query {
            user: self,
            rows: self.ls_fund()?,
        })
    }
    pub fn currencies(&self) -> Result<Query<'_, Currency>, UserError> {
        Ok(Query {
            user: self,
            rows: self.ls_currency()?,
        })
    }

    /// Renders a structured CLI table to standard output for visual reporting.
    fn print_table<T: Display>(&self, title: &str, headers: &[&str], widths: &[usize], rows: &[T]) {
        let no_width = rows.len().to_string().len().max(2); // at least 2 for "NO"

        let sep: String = format!("+{}", "-".repeat(no_width + 2))
            + &widths
                .iter()
                .map(|w| format!("+{}", "-".repeat(w + 2)))
                .collect::<String>()
            + "+";

        let header: String = format!("| {:<no_width$} ", "NO")
            + &headers
                .iter()
                .zip(widths.iter())
                .map(|(h, w)| format!("| {:<w$} ", h))
                .collect::<String>()
            + "|";

        println!();
        println!("> {title}");
        println!("{sep}");
        println!("{header}");
        println!("{sep}");
        for (i, row) in rows.iter().enumerate() {
            println!("| {:<no_width$} | {row} |", i + 1);
        }
        println!("{sep}");
        println!();
    }

    fn print_table_with_link<T: Display>(
        &self,
        title: &str,
        headers: &[&str],
        widths: &[usize],
        rows: &[T],
        link_header: &str,
        link_labels: &[String],
    ) {
        let no_width = rows.len().to_string().len().max(2);

        // dynamic, like no_width — based on actual content + header label
        let link_width = link_labels
            .iter()
            .map(|s| s.len())
            .chain(std::iter::once(link_header.len()))
            .max()
            .unwrap_or(link_header.len());

        let sep: String = format!("+{}", "-".repeat(no_width + 2))
            + &widths
                .iter()
                .map(|w| format!("+{}", "-".repeat(w + 2)))
                .collect::<String>()
            + &format!("+{}", "-".repeat(link_width + 2))
            + "+";

        let header: String = format!("| {:<no_width$} ", "NO")
            + &headers
                .iter()
                .zip(widths.iter())
                .map(|(h, w)| format!("| {:<w$} ", h))
                .collect::<String>()
            + &format!("| {:<link_width$} ", link_header)
            + "|";

        println!();
        println!("> {title}");
        println!("{sep}");
        println!("{header}");
        println!("{sep}");
        for (i, (row, link)) in rows.iter().zip(link_labels.iter()).enumerate() {
            println!("| {:<no_width$} | {row} | {:<link_width$} |", i + 1, link);
        }
        println!("{sep}");
        println!();
    }
}

/// Errors related to data processing constraints and index range checks.
#[derive(Error, Debug)]
pub enum InputError {
    #[error("Failed to create directory: {0}.")]
    InvalidDir(String),

    #[error("{0} already exists but as a different category type.")]
    WrongVariant(String),

    #[error("No item at index {0}.")]
    InvalidIndex(usize),

    #[error("{0} already exists in {1}")]
    ExistingItem(String, String),

    #[error(
        "Category used by {0} transaction(s), restricted from editing variant. Use 'force_edit_variant' to unlink all transactions for this category and edit."
    )]
    CategoryInUse(usize),
}

/// Top-level error enum wrapping specialized subsystem failure states.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Error, Debug)]
pub enum UserError {
    #[error(transparent)]
    Input(#[from] InputError),

    #[error(transparent)]
    Date(#[from] DateError),

    #[error(transparent)]
    SQL(#[from] rusqlite::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constructs an isolated sandbox instance using memory storage for testing routines.
    fn setup() -> User {
        User::new_in_memory("test").unwrap()
    }

    #[test]
    fn add_group_persists() {
        let user = setup();
        user.add_group("Food").unwrap();
        let groups = user.ls_group().unwrap(); // private, accessible here
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].label.name, "Food");
    }

    #[test]
    fn paired_transaction_links_cross_reference() {
        let user = setup();
        user.add_currency("MYR")
            .unwrap()
            .add_group("Transfer")
            .unwrap()
            .add_paired_category("Internal")
            .unwrap()
            .add_fund("Checking")
            .unwrap()
            .add_fund("Savings")
            .unwrap();
        user.add_paired_transaction(
            "Move",
            None,
            (50000, "MYR"),
            (50000, "MYR"),
            (1, 6, 2026),
            "Transfer",
            "Internal",
            "Checking",
            "Savings",
        )
        .unwrap();

        let transactions = user.ls_transaction().unwrap();
        assert_eq!(transactions.len(), 2);
        let ids: Vec<_> = transactions.iter().map(|t| t.label.id).collect();
        let links: Vec<_> = transactions.iter().map(|t| t.link.unwrap()).collect();

        // Confirm mutual reference constraints match up cleanly across records
        assert!(ids.contains(&links[0]));
        assert!(ids.contains(&links[1]));
        assert_ne!(links[0], links[1]);
    }
}
