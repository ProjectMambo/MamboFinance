// Imports from internal user module
use crate::user::{HasLabel, Label};
use std::fmt::{Display, Formatter};

/// Represents a financial currency asset descriptor used for monetary valuation.
#[derive(Clone, Debug)]
pub struct Currency {
    pub label: Label,
    pub count: usize,
}

impl Currency {
    /// Maps a single SQLite row to a `Currency` instance starting from the base index.
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Self::from_row_offset(row, 0)
    }

    /// Maps a single SQLite row to a `Currency` instance using a specified column offset.
    ///
    /// Pulls identity fields from the record layout without evaluating description parameters.
    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Currency {
            label: Label::from_row_offset_no_desc(row, offset)?,
            count: row.get::<_, i64>(offset + 2)? as usize,
        })
    }
}

impl Display for Currency {
    /// Formats the currency information, supporting alternate styling layouts if flagged.
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            return write!(f, "{:#}", self.label);
        }

        write!(f, "{}", self.label)
    }
}

impl HasLabel for Currency {
    fn label(&self) -> &Label {
        &self.label
    }

    fn table() -> &'static str {
        "currencies"
    }
}

impl PartialEq for Currency {
    /// Compares two currencies solely by their associated lexical name values.
    fn eq(&self, other: &Self) -> bool {
        self.label.name == other.label.name
    }
}

// region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // region: helpers

    // Builds an in-memory connection seeded with a single currency-shaped row
    // (id, name, transaction_count) to exercise the row-mapping constructors.
    fn connection_with_currency_row(name: &str, count: i64) -> Connection {
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE currencies (id BLOB PRIMARY KEY, name TEXT, transaction_count INTEGER);",
            (),
        )
        .expect("failed to create table");

        let id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO currencies (id, name, transaction_count) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, name, count],
        )
        .expect("failed to insert row");

        conn
    }

    // endregion

    // region: Currency::from_row

    #[test]
    fn from_row_maps_name_and_count_from_base_index() {
        // Arrange
        let conn = connection_with_currency_row("US Dollar", 5);

        // Act
        let currency: Currency = conn
            .query_row(
                "SELECT id, name, transaction_count FROM currencies",
                (),
                Currency::from_row,
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(currency.label.name, "US Dollar");
        assert_eq!(currency.count, 5);
    }

    #[test]
    fn from_row_does_not_populate_description() {
        // Arrange
        let conn = connection_with_currency_row("US Dollar", 0);

        // Act
        let currency: Currency = conn
            .query_row(
                "SELECT id, name, transaction_count FROM currencies",
                (),
                Currency::from_row,
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(currency.label.description, None);
    }

    // endregion

    // region: Currency::from_row_offset

    #[test]
    fn from_row_offset_respects_a_nonzero_column_offset() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE currencies (padding INTEGER, id BLOB, name TEXT, transaction_count INTEGER);",
            (),
        )
        .expect("failed to create table");
        let id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO currencies (padding, id, name, transaction_count) VALUES (1, ?1, ?2, ?3)",
            rusqlite::params![id, "Euro", 3i64],
        )
        .expect("failed to insert row");

        // Act
        let currency: Currency = conn
            .query_row(
                "SELECT padding, id, name, transaction_count FROM currencies",
                (),
                |row| Currency::from_row_offset(row, 1),
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(currency.label.name, "Euro");
        assert_eq!(currency.count, 3);
    }

    // endregion

    // region: Display for Currency

    #[test]
    fn display_default_renders_padded_name_like_label() {
        // Arrange
        let currency = Currency {
            label: Label::new("USD", None),
            count: 0,
        };

        // Act
        let rendered = format!("{}", currency);

        // Assert
        assert_eq!(rendered, format!("{}", currency.label));
    }

    #[test]
    fn display_alternate_delegates_to_label_alternate_format() {
        // Arrange
        let currency = Currency {
            label: Label::new("USD", Some("US Dollar")),
            count: 0,
        };

        // Act
        let rendered = format!("{:#}", currency);

        // Assert
        assert_eq!(rendered, format!("{:#}", currency.label));
        assert!(rendered.contains("US Dollar"));
    }

    // endregion

    // region: HasLabel for Currency

    #[test]
    fn has_label_label_returns_the_underlying_label() {
        // Arrange
        let currency = Currency {
            label: Label::new("USD", None),
            count: 0,
        };

        // Act
        let label_ref = currency.label();

        // Assert
        assert_eq!(label_ref.name, "USD");
    }

    #[test]
    fn has_label_table_returns_currencies() {
        // Arrange
        // Act
        let table = Currency::table();

        // Assert
        assert_eq!(table, "currencies");
    }

    // endregion

    // region: PartialEq for Currency

    #[test]
    fn equality_holds_for_matching_names_regardless_of_count() {
        // Arrange
        let a = Currency {
            label: Label::new("USD", None),
            count: 1,
        };
        let b = Currency {
            label: Label::new("USD", None),
            count: 99,
        };

        // Act
        let is_equal = a == b;

        // Assert
        assert!(is_equal);
    }

    #[test]
    fn equality_fails_for_different_names() {
        // Arrange
        let a = Currency {
            label: Label::new("USD", None),
            count: 0,
        };
        let b = Currency {
            label: Label::new("EUR", None),
            count: 0,
        };

        // Act
        let is_equal = a == b;

        // Assert
        assert!(!is_equal);
    }

    // endregion
}

// endregion
