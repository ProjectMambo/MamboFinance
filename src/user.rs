mod category;
mod currency;
mod fund;
mod group;
mod transaction;
mod types;

use category::*;
use currency::*;
use fund::*;
use group::*;
use transaction::*;
use types::*;

use rusqlite::{Connection, Result};
use std::fmt::Display;
use std::fs;
use thiserror::Error;
use uuid::Uuid;

use crate::user::InputError::WrongVariant;

pub const NAME_LIMIT: usize = 25;
pub const DESC_LIMIT: usize = 25;
pub const AMOUNT_LIMIT: usize = 10;
pub const VARIANT_LIMIT: usize = 8;

#[allow(dead_code)]
pub struct User {
    label: Label,
    conn: Connection,
}

impl User {
    pub fn new(name: &str) -> Result<Self, UserError> {
        let folder_name = "storage";
        fs::create_dir_all(folder_name)
            .map_err(|e| UserError::Input(InputError::InvalidDir(format!("{}", e))))?;
        let db_path = format!("{}/{}.db", folder_name, name);
        let conn = Connection::open(&db_path).map_err(UserError::SQL)?;

        conn.execute("PRAGMA foreign_keys = ON;", ())
            .map_err(UserError::SQL)?;

        let table_queries = [
            "CREATE TABLE IF NOT EXISTS transactions (
            id BLOB PRIMARY KEY,
            name TEXT,
            description TEXT,
            amount INTEGER,
            currency_id BLOB REFERENCES currencies(id),
            day INTEGER,
            month INTEGER,
            year INTEGER, 
            group_id BLOB REFERENCES groups(id),
            category_id BLOB REFERENCES categories(id),
            fund_id BLOB REFERENCES funds(id),
            link_id BLOB REFERENCES transactions(id)
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

        tx.execute("PRAGMA defer_foreign_keys = ON;", ())?;

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

    pub fn add_group(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table(name, &TableTarget::GROUP, &label, |conn, label| {
            conn.execute(
                "INSERT INTO groups (id, name) VALUES (?1, ?2)",
                rusqlite::params![label.id, label.name],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }

    pub fn add_category(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table(name, &TableTarget::CATEGORY, &label, |conn, label| {
            conn.execute(
                "INSERT INTO categories (id, name, variant) VALUES (?1, ?2, ?3)",
                rusqlite::params![label.id, label.name, 0],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }
    pub fn add_paired_category(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table(name, &TableTarget::CATEGORY, &label, |conn, label| {
            conn.execute(
                "INSERT INTO categories (id, name, variant) VALUES (?1, ?2, ?3)",
                rusqlite::params![label.id, label.name, 1],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }

    pub fn add_fund(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table(name, &TableTarget::FUND, &label, |conn, label| {
            conn.execute(
                "INSERT INTO funds (id, name) VALUES (?1, ?2)",
                rusqlite::params![label.id, label.name],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }

    pub fn add_currency(&self, name: &str) -> Result<&Self, UserError> {
        let label = Label::new(name, None);
        self.add_unique_to_table(name, &TableTarget::CURRENCY, &label, |conn, label| {
            conn.execute(
                "INSERT INTO currencies (id, name) VALUES (?1, ?2)",
                rusqlite::params![label.id, label.name],
            )
            .map(|_| ())
        })?;
        Ok(self)
    }

    fn add_unique_to_table(
        &self,
        name: &str,
        table: &TableTarget,
        label: &Label,
        insert: impl FnOnce(&rusqlite::Connection, &Label) -> rusqlite::Result<()>,
    ) -> Result<(), UserError> {
        let existing = self.get_from_table(name, table);
        match existing {
            Ok(_) => Ok(()),
            Err(UserError::SQL(rusqlite::Error::QueryReturnedNoRows)) => {
                insert(&self.conn, label).map_err(UserError::SQL)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn add_to_table(
        &self,
        label: &Label,
        insert: impl FnOnce(&rusqlite::Connection, &Label) -> rusqlite::Result<()>,
    ) -> Result<(), UserError> {
        insert(&self.conn, label).map_err(UserError::SQL)
    }
}

impl User {
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
        self.get_from_table(name, &TableTarget::GROUP)
    }

    fn get_category(&self, name: &str) -> Result<Uuid, UserError> {
        self.get_from_table(name, &TableTarget::CATEGORY)
    }

    fn get_fund(&self, name: &str) -> Result<Uuid, UserError> {
        self.get_from_table(name, &TableTarget::FUND)
    }

    fn get_currency(&self, name: &str) -> Result<Uuid, UserError> {
        self.get_from_table(name, &TableTarget::CURRENCY)
    }

    fn get_from_table(&self, name: &str, table: &TableTarget) -> Result<Uuid, UserError> {
        let query = format!("SELECT id FROM {} WHERE name = ?1", table.name());
        self.conn
            .query_row(&query, [name], |row| row.get(0))
            .map_err(UserError::SQL)
    }
}

impl User {
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
        self.ls_table(&TableTarget::GROUP, Group::from_row)
    }

    fn ls_category(&self) -> Result<Vec<Category>, UserError> {
        self.ls_table(&TableTarget::CATEGORY, Category::from_row)
    }

    fn ls_fund(&self) -> Result<Vec<Fund>, UserError> {
        self.ls_table(&TableTarget::FUND, Fund::from_row)
    }

    fn ls_currency(&self) -> Result<Vec<Currency>, UserError> {
        self.ls_table(&TableTarget::CURRENCY, Currency::from_row)
    }

    fn ls_table<T>(
        &self,
        table: &TableTarget,
        from_row: impl Fn(&rusqlite::Row) -> rusqlite::Result<T>,
    ) -> Result<Vec<T>, UserError> {
        let query = format!("SELECT * FROM {}", table.name());
        let mut stmt = self.conn.prepare(&query).map_err(UserError::SQL)?;
        let rows = stmt.query_map([], from_row).map_err(UserError::SQL)?;
        rows.collect::<rusqlite::Result<Vec<T>>>()
            .map_err(UserError::SQL)
    }

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
    pub fn print_transaction(&self) -> Result<&Self, UserError> {
        let rows = self.ls_transaction()?;
        self.print_table(
            "TRANSACTION",
            &[
                "NAME",
                "DESCRIPTION",
                "AMOUNT",
                "DATE",
                "GROUP",
                "CATEGORY",
                "FUND",
            ],
            &[
                NAME_LIMIT,
                DESC_LIMIT,
                AMOUNT_LIMIT + 13,
                11,
                NAME_LIMIT,
                NAME_LIMIT,
                NAME_LIMIT,
            ],
            &rows,
        );
        Ok(self)
    }

    pub fn print_group(&self) -> Result<&Self, UserError> {
        let rows = self.ls_group()?;
        self.print_table("GROUP", &["NAME"], &[NAME_LIMIT], &rows);
        Ok(self)
    }

    pub fn print_category(&self) -> Result<&Self, UserError> {
        let rows = self.ls_category()?;
        self.print_table(
            "CATEGORY",
            &["NAME", "VARIANT"],
            &[NAME_LIMIT, VARIANT_LIMIT],
            &rows,
        );
        Ok(self)
    }

    pub fn print_fund(&self) -> Result<&Self, UserError> {
        let rows = self.ls_fund()?;
        self.print_table("FUND", &["NAME"], &[NAME_LIMIT], &rows);
        Ok(self)
    }

    pub fn print_currency(&self) -> Result<&Self, UserError> {
        let rows = self.ls_currency()?;
        self.print_table("CURRENCY", &["NAME"], &[NAME_LIMIT], &rows);
        Ok(self)
    }

    fn print_table<T: Display>(&self, title: &str, headers: &[&str], widths: &[usize], rows: &[T]) {
        let sep: String = widths
            .iter()
            .map(|w| format!("+{}", "-".repeat(w + 2)))
            .collect::<String>()
            + "+";

        let header: String = headers
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
        for row in rows {
            println!("| {row} |");
        }
        println!("{sep}");
        println!();
    }
}

#[derive(Error, Debug)]
pub enum InputError {
    #[error("Failed to create directory: {0}.")]
    InvalidDir(String),
    #[error("{0} already exists but as a different category type.")]
    WrongVariant(String),
}

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

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug)]
enum TableTarget {
    GROUP,
    CATEGORY,
    FUND,
    CURRENCY,
}

impl TableTarget {
    fn name(&self) -> &'static str {
        match self {
            TableTarget::GROUP => "groups",
            TableTarget::CATEGORY => "categories",
            TableTarget::FUND => "funds",
            TableTarget::CURRENCY => "currencies",
        }
    }
}
