// Imports from internal user module
use crate::user::{HasLabel, Label, VARIANT_LIMIT};
use std::fmt::{Display, Formatter};

/// Represents a transaction classification group which determines structural rules.
#[derive(Clone)]
pub struct Category {
    pub label: Label,
    pub variant: CategoryVariant,
    pub count: usize,
}

impl Category {
    /// Maps a single SQLite row to a `Category` instance starting from the base index.
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Self::from_row_offset(row, 0)
    }

    /// Maps a single SQLite row to a `Category` instance using a specified column offset.
    ///
    /// Pulls identity parameters first, then extracts the integer representation of the structural variant.
    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Category {
            label: Label::from_row_offset_no_desc(row, offset)?,
            variant: row.get(offset + 2)?,
            count: row.get::<_, i64>(offset + 3)? as usize,
        })
    }
}

impl Display for Category {
    /// Formats the category metadata, altering output formatting parameters based on precision flags.
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            return write!(f, "{}", self.label);
        }

        write!(
            f,
            "{} | {:<width$}",
            self.label,
            format!("{:?}", self.variant),
            width = VARIANT_LIMIT,
        )
    }
}

impl HasLabel for Category {
    fn label(&self) -> &Label {
        &self.label
    }

    fn table() -> &'static str {
        "categories"
    }
}

/// Structural variance options classifying standard vs multi-entry double transactions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CategoryVariant {
    Single = 0,
    Paired = 1,
}

impl rusqlite::types::FromSql for CategoryVariant {
    /// Maps the storage engine's small integer representation back into the local Rust enum variant.
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let int_value = value.as_i64()?;
        match int_value {
            0 => Ok(CategoryVariant::Single),
            1 => Ok(CategoryVariant::Paired),
            _ => Err(rusqlite::types::FromSqlError::OutOfRange(int_value)),
        }
    }
}

impl rusqlite::types::ToSql for CategoryVariant {
    /// Serializes the local runtime state enum safely as an integer matching database type requirements.
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(*self as i64))
    }
}

// region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use rusqlite::types::{FromSql, ToSql, ValueRef};

    // region: helpers

    // Builds an in-memory connection seeded with a single category-shaped row
    // (id, name, variant, transaction_count) to exercise the row-mapping constructors.
    fn connection_with_category_row(name: &str, variant: i64, count: i64) -> Connection {
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE categories (id BLOB PRIMARY KEY, name TEXT, variant INTEGER, transaction_count INTEGER);",
            (),
        )
        .expect("failed to create table");

        let id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO categories (id, name, variant, transaction_count) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, name, variant, count],
        )
        .expect("failed to insert row");

        conn
    }

    // endregion

    // region: Category::from_row

    #[test]
    fn from_row_maps_name_variant_and_count() {
        // Arrange
        let conn = connection_with_category_row("Salary", 1, 4);

        // Act
        let category: Category = conn
            .query_row(
                "SELECT id, name, variant, transaction_count FROM categories",
                (),
                Category::from_row,
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(category.label.name, "Salary");
        assert_eq!(category.variant, CategoryVariant::Paired);
        assert_eq!(category.count, 4);
    }

    #[test]
    fn from_row_maps_single_variant_correctly() {
        // Arrange
        let conn = connection_with_category_row("Groceries", 0, 0);

        // Act
        let category: Category = conn
            .query_row(
                "SELECT id, name, variant, transaction_count FROM categories",
                (),
                Category::from_row,
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(category.variant, CategoryVariant::Single);
    }

    // endregion

    // region: Category::from_row_offset

    #[test]
    fn from_row_offset_respects_a_nonzero_column_offset() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE categories (padding INTEGER, id BLOB, name TEXT, variant INTEGER, transaction_count INTEGER);",
            (),
        )
        .expect("failed to create table");
        let id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO categories (padding, id, name, variant, transaction_count) VALUES (1, ?1, ?2, ?3, ?4)",
            rusqlite::params![id, "Transfer", 1i64, 2i64],
        )
        .expect("failed to insert row");

        // Act
        let category: Category = conn
            .query_row(
                "SELECT padding, id, name, variant, transaction_count FROM categories",
                (),
                |row| Category::from_row_offset(row, 1),
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(category.label.name, "Transfer");
        assert_eq!(category.variant, CategoryVariant::Paired);
        assert_eq!(category.count, 2);
    }

    // endregion

    // region: Display for Category

    #[test]
    fn display_default_includes_label_and_variant_debug_text() {
        // Arrange
        let category = Category {
            label: Label::new("Groceries", None),
            variant: CategoryVariant::Single,
            count: 0,
        };

        // Act
        let rendered = format!("{}", category);

        // Assert
        assert!(rendered.contains("Groceries"));
        assert!(rendered.contains("Single"));
        assert!(rendered.contains('|'));
    }

    #[test]
    fn display_alternate_renders_label_only_without_variant() {
        // Arrange
        let category = Category {
            label: Label::new("Groceries", None),
            variant: CategoryVariant::Paired,
            count: 0,
        };

        // Act
        let rendered = format!("{:#}", category);

        // Assert
        assert_eq!(rendered, "Groceries");
        assert!(!rendered.contains("Paired"));
    }

    // endregion

    // region: HasLabel for Category

    #[test]
    fn has_label_label_returns_the_underlying_label() {
        // Arrange
        let category = Category {
            label: Label::new("Groceries", None),
            variant: CategoryVariant::Single,
            count: 0,
        };

        // Act
        let label_ref = category.label();

        // Assert
        assert_eq!(label_ref.name, "Groceries");
    }

    #[test]
    fn has_label_table_returns_categories() {
        // Arrange
        // Act
        let table = Category::table();

        // Assert
        assert_eq!(table, "categories");
    }

    // endregion

    // region: CategoryVariant FromSql

    #[test]
    fn from_sql_maps_zero_to_single() {
        // Arrange
        let value = ValueRef::Integer(0i64);

        // Act
        let result = CategoryVariant::column_result(value);

        // Assert
        assert_eq!(result.unwrap(), CategoryVariant::Single);
    }

    #[test]
    fn from_sql_maps_one_to_paired() {
        // Arrange
        let value = ValueRef::Integer(1i64);

        // Act
        let result = CategoryVariant::column_result(value);

        // Assert
        assert_eq!(result.unwrap(), CategoryVariant::Paired);
    }

    #[test]
    fn from_sql_rejects_out_of_range_integer() {
        // Arrange
        let value = ValueRef::Integer(7i64);

        // Act
        let result = CategoryVariant::column_result(value);

        // Assert
        assert!(result.is_err());
    }

    // endregion

    // region: CategoryVariant ToSql

    #[test]
    fn to_sql_serializes_single_as_zero() {
        // Arrange
        let variant = CategoryVariant::Single;

        // Act
        let output = variant.to_sql().expect("should serialize");

        // Assert
        match output {
            rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer(v)) => {
                assert_eq!(v, 0);
            }
            other => panic!("unexpected ToSqlOutput variant: {:?}", other),
        }
    }

    #[test]
    fn to_sql_serializes_paired_as_one() {
        // Arrange
        let variant = CategoryVariant::Paired;

        // Act
        let output = variant.to_sql().expect("should serialize");

        // Assert
        match output {
            rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer(v)) => {
                assert_eq!(v, 1);
            }
            other => panic!("unexpected ToSqlOutput variant: {:?}", other),
        }
    }

    #[test]
    fn to_sql_and_from_sql_round_trip_through_a_real_connection() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute("CREATE TABLE variant_test (variant INTEGER);", ())
            .expect("failed to create table");
        conn.execute(
            "INSERT INTO variant_test (variant) VALUES (?1)",
            [CategoryVariant::Paired],
        )
        .expect("failed to insert row");

        // Act
        let result: CategoryVariant = conn
            .query_row("SELECT variant FROM variant_test", (), |row| row.get(0))
            .expect("query should succeed");

        // Assert
        assert_eq!(result, CategoryVariant::Paired);
    }

    // endregion
}

// endregion
