// Imports from internal user module
use crate::user::{Flattenable, HasLabel, Label};
use std::fmt::{Display, Formatter};

/// Represents an asset storage account or location (e.g., Cash, Bank, Savings) within the system.
#[derive(Clone)]
pub struct Fund {
    /// Associated metadata label containing name information.
    pub label: Label,
    /// Historical count of transaction assignments.
    pub count: usize,
}

impl Fund {
    /// Maps a single SQLite row to a `Fund` instance starting from the base index.
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Self::from_row_offset(row, 0)
    }

    /// Maps a single SQLite row to a `Fund` instance using a specified column offset.
    ///
    /// Pulls identity fields from the record layout without evaluating description parameters.
    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Fund {
            label: Label::from_row_offset_no_desc(row, offset)?,
            count: row.get::<_, i64>(offset + 2)? as usize,
        })
    }
}

impl Display for Fund {
    /// Formats the fund data using its underlying label configuration.
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl HasLabel for Fund {
    /// Returns a reference to the underlying structural `Label`.
    fn label(&self) -> &Label {
        &self.label
    }

    /// Declares the corresponding database source table identifier string.
    fn table() -> &'static str {
        "funds"
    }
}

impl Flattenable for Fund {
    /// Flattens categorical tracking vectors into raw field vector elements.
    fn flatten(&self) -> Vec<String> {
        vec![self.label.to_string()]
    }
}

// region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // region: helpers

    /// Builds an in-memory connection seeded with a single fund-shaped row to exercise the row-mapping constructors.
    fn connection_with_fund_row(name: &str, count: i64) -> Connection {
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE funds (id BLOB PRIMARY KEY, name TEXT, transaction_count INTEGER);",
            (),
        )
        .expect("failed to create table");

        let id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO funds (id, name, transaction_count) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, name, count],
        )
        .expect("failed to insert row");

        conn
    }

    // endregion

    // region: Fund::from_row

    /// Verifies that standard row translation parses field names and tracking allocations accurately.
    #[test]
    fn from_row_maps_name_and_count() {
        // Arrange
        let conn = connection_with_fund_row("Cash", 3);

        // Act
        let fund: Fund = conn
            .query_row(
                "SELECT id, name, transaction_count FROM funds",
                (),
                Fund::from_row,
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(fund.label.name, "Cash");
        assert_eq!(fund.count, 3);
    }

    /// Verifies that record parsing tracks optional descriptions as default empty elements.
    #[test]
    fn from_row_does_not_populate_description() {
        // Arrange
        let conn = connection_with_fund_row("Cash", 0);

        // Act
        let fund: Fund = conn
            .query_row(
                "SELECT id, name, transaction_count FROM funds",
                (),
                Fund::from_row,
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(fund.label.description, None);
    }

    // endregion

    // region: Fund::from_row_offset

    /// Verifies row extraction processes tracking fields accurately given a leading database column offset index.
    #[test]
    fn from_row_offset_respects_a_nonzero_column_offset() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE funds (padding INTEGER, id BLOB, name TEXT, transaction_count INTEGER);",
            (),
        )
        .expect("failed to create table");
        let id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO funds (padding, id, name, transaction_count) VALUES (1, ?1, ?2, ?3)",
            rusqlite::params![id, "Bank", 1i64],
        )
        .expect("failed to insert row");

        // Act
        let fund: Fund = conn
            .query_row(
                "SELECT padding, id, name, transaction_count FROM funds",
                (),
                |row| Fund::from_row_offset(row, 1),
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(fund.label.name, "Bank");
        assert_eq!(fund.count, 1);
    }

    // endregion

    // region: Display for Fund

    /// Verifies that standard formatting strings cleanly map down to nested property blocks.
    #[test]
    fn display_delegates_to_underlying_label_format() {
        // Arrange
        let fund = Fund {
            label: Label::new("Cash", None),
            count: 0,
        };

        // Act
        let rendered = format!("{}", fund);

        // Assert
        assert_eq!(rendered, format!("{}", fund.label));
    }

    // endregion

    // region: HasLabel for Fund

    /// Verifies trait signatures route accurately to internal label properties.
    #[test]
    fn has_label_label_returns_the_underlying_label() {
        // Arrange
        let fund = Fund {
            label: Label::new("Cash", None),
            count: 0,
        };

        // Act
        let label_ref = fund.label();

        // Assert
        assert_eq!(label_ref.name, "Cash");
    }

    /// Verifies trait mappings match the designated collection target labels.
    #[test]
    fn has_label_table_returns_funds() {
        // Arrange & Act
        let table = Fund::table();

        // Assert
        assert_eq!(table, "funds");
    }

    // endregion
}

// endregion
