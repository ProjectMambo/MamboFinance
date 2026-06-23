// Imports from internal user module
use crate::user::{Flattenable, HasLabel, Label};
use std::fmt::{Display, Formatter};

/// Represents a financial currency asset descriptor used for monetary valuation.
#[derive(Clone, Debug)]
pub struct Currency {
    /// Associated metadata label containing name information.
    pub label: Label,
    /// Historical count of transaction assignments.
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
    /// Returns a reference to the underlying structural `Label`.
    fn label(&self) -> &Label {
        &self.label
    }

    /// Declares the corresponding database source table identifier string.
    fn table() -> &'static str {
        "currencies"
    }
}

impl Flattenable for Currency {
    /// Flattens categorical tracking vectors into raw field vector elements.
    fn flatten(&self) -> Vec<String> {
        vec![self.label.to_string()]
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

    /// Builds an in-memory connection seeded with a single currency-shaped row to exercise the row-mapping constructors.
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

    /// Verifies that standard row translation parses field names and tracking allocations accurately.
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

    /// Verifies that record parsing tracks optional descriptions as default empty elements.
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

    /// Verifies row extraction processes tracking fields accurately given a leading database column offset index.
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

    /// Verifies that standard formatting strings cleanly map down to nested property blocks.
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

    /// Verifies that precision styling decorators safely propagate down to underlying components.
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

    /// Verifies trait signatures route accurately to internal label properties.
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

    /// Verifies trait mappings match the designated collection target labels.
    #[test]
    fn has_label_table_returns_currencies() {
        // Arrange & Act
        let table = Currency::table();

        // Assert
        assert_eq!(table, "currencies");
    }

    // endregion

    // region: PartialEq for Currency

    /// Verifies asset evaluation operations derive equivalence from structural labels instead of balance counters.
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

    /// Verifies differing identity descriptors fail relative matching queries.
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
