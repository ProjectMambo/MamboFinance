// Imports from internal user module
use crate::user::{Flattenable, HasLabel, Label};
use std::fmt::{Display, Formatter};

/// Represents a structural category classification used to organize ledger operations.
#[derive(Clone)]
pub struct Group {
    /// Associated metadata label containing name information.
    pub label: Label,
    /// Historical count of transaction assignments.
    pub count: usize,
}

impl Group {
    /// Maps a single SQLite row to a `Group` instance starting from the base index.
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Self::from_row_offset(row, 0)
    }

    /// Maps a single SQLite row to a `Group` instance using a specified column offset.
    ///
    /// Pulls descriptive fields from the record layout without evaluating description parameters.
    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Group {
            label: Label::from_row_offset_no_desc(row, offset)?,
            count: row.get::<_, i64>(offset + 2)? as usize,
        })
    }
}

impl Display for Group {
    /// Formats the transaction group data using its clean underlying label configuration.
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl HasLabel for Group {
    /// Returns a reference to the underlying structural `Label`.
    fn label(&self) -> &Label {
        &self.label
    }

    /// Declares the corresponding database source table identifier string.
    fn table() -> &'static str {
        "groups"
    }
}

impl Flattenable for Group {
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

    /// Builds an in-memory connection seeded with a single group-shaped row to exercise the row-mapping constructors.
    fn connection_with_group_row(name: &str, count: i64) -> Connection {
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE groups (id BLOB PRIMARY KEY, name TEXT, transaction_count INTEGER);",
            (),
        )
        .expect("failed to create table");

        let id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO groups (id, name, transaction_count) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, name, count],
        )
        .expect("failed to insert row");

        conn
    }

    // endregion

    // region: Group::from_row

    /// Verifies that standard row translation parses field names and tracking allocations accurately.
    #[test]
    fn from_row_maps_name_and_count() {
        // Arrange
        let conn = connection_with_group_row("Personal", 7);

        // Act
        let group: Group = conn
            .query_row(
                "SELECT id, name, transaction_count FROM groups",
                (),
                Group::from_row,
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(group.label.name, "Personal");
        assert_eq!(group.count, 7);
    }

    /// Verifies that record parsing tracks optional descriptions as default empty elements.
    #[test]
    fn from_row_does_not_populate_description() {
        // Arrange
        let conn = connection_with_group_row("Personal", 0);

        // Act
        let group: Group = conn
            .query_row(
                "SELECT id, name, transaction_count FROM groups",
                (),
                Group::from_row,
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(group.label.description, None);
    }

    // endregion

    // region: Group::from_row_offset

    /// Verifies row extraction processes tracking fields accurately given a leading database column offset index.
    #[test]
    fn from_row_offset_respects_a_nonzero_column_offset() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE groups (padding INTEGER, id BLOB, name TEXT, transaction_count INTEGER);",
            (),
        )
        .expect("failed to create table");
        let id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO groups (padding, id, name, transaction_count) VALUES (1, ?1, ?2, ?3)",
            rusqlite::params![id, "Business", 2i64],
        )
        .expect("failed to insert row");

        // Act
        let group: Group = conn
            .query_row(
                "SELECT padding, id, name, transaction_count FROM groups",
                (),
                |row| Group::from_row_offset(row, 1),
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(group.label.name, "Business");
        assert_eq!(group.count, 2);
    }

    // endregion

    // region: Display for Group

    /// Verifies that standard formatting strings cleanly map down to nested property blocks.
    #[test]
    fn display_delegates_to_underlying_label_format() {
        // Arrange
        let group = Group {
            label: Label::new("Personal", None),
            count: 0,
        };

        // Act
        let rendered = format!("{}", group);

        // Assert
        assert_eq!(rendered, format!("{}", group.label));
    }

    // endregion

    // region: HasLabel for Group

    /// Verifies trait signatures route accurately to internal label properties.
    #[test]
    fn has_label_label_returns_the_underlying_label() {
        // Arrange
        let group = Group {
            label: Label::new("Personal", None),
            count: 0,
        };

        // Act
        let label_ref = group.label();

        // Assert
        assert_eq!(label_ref.name, "Personal");
    }

    /// Verifies trait mappings match the designated collection target labels.
    #[test]
    fn has_label_table_returns_groups() {
        // Arrange & Act
        let table = Group::table();

        // Assert
        assert_eq!(table, "groups");
    }

    // endregion
}

// endregion
