// Imports from internal user module
use crate::user::Flattenable;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

/// Metadata container providing identity, naming, and description for database entities.
#[derive(Clone, Debug)]
pub struct Label {
    /// Unique identifier for the entity.
    pub id: Uuid,
    /// Sanitized, title-cased name.
    pub name: String,
    /// Optional detailed description.
    pub description: Option<String>,
}

impl Label {
    /// Creates a new `Label` with a random V4 UUID and a formatted title-case name.
    pub fn new(name: &str, description: Option<&str>) -> Self {
        let id = Uuid::new_v4();
        let des = description.map(String::from);

        Self {
            id,
            name: Label::fmt(name),
            description: des,
        }
    }

    /// Deserializes a `Label` from a database row using the specified column index offset.
    /// Expects `id`, `name`, and `description` to be sequential from the offset.
    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Label {
            id: row.get(offset)?,
            name: row.get(offset + 1)?,
            description: row.get(offset + 2)?,
        })
    }

    /// Deserializes a `Label` from a database row, skipping the description column
    /// and defaulting it to `None`.
    pub fn from_row_offset_no_desc(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Label {
            id: row.get(offset)?,
            name: row.get(offset + 1)?,
            description: None,
        })
    }

    /// Normalizes delimiter-separated strings (spaces, underscores, hyphens) into Title Case.
    ///
    /// # Examples
    /// "my_test-string" becomes "My Test String".
    pub fn fmt(input: &str) -> String {
        let delimiters = " _-";
        input
            .split(|c| delimiters.contains(c))
            .filter(|s| !s.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl Display for Label {
    /// Formats the label as "Name | Description" if a description exists, otherwise just "Name".
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.description {
            Some(desc) => write!(f, "{} | {}", self.name, desc),
            None => write!(f, "{}", self.name),
        }
    }
}

impl Flattenable for Label {
    /// Flattens the label struct fields into a vector of strings.
    fn flatten(&self) -> Vec<String> {
        vec![
            self.name.to_string(),
            self.description
                .as_ref()
                .map_or_else(String::new, |o| o.to_string()),
        ]
    }
}

/// Defines a standard interface for types that contain or wrap a `Label`.
pub trait HasLabel {
    /// Returns the name associated with the entity label.
    fn name(&self) -> &str {
        &self.label().name
    }

    /// Returns the unique identifier from the entity label.
    fn id(&self) -> Uuid {
        self.label().id
    }

    /// Borrows the underlying `Label` struct.
    fn label(&self) -> &Label;

    /// Returns the database table name corresponding to the entity.
    fn table() -> &'static str;
}

// region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // region: helpers

    /// Generates an in-memory SQLite connection containing a mock items table for testing.
    fn connection_with_label_row(name: &str, description: Option<&str>) -> Connection {
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE items (id BLOB PRIMARY KEY, name TEXT, description TEXT);",
            (),
        )
        .expect("failed to create table");

        let id = Uuid::new_v4();
        conn.execute(
            "INSERT INTO items (id, name, description) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, name, description],
        )
        .expect("failed to insert row");

        conn
    }

    /// Concrete mock struct used to validate default trait implementations of `HasLabel`.
    struct DummyEntity {
        label: Label,
    }

    impl HasLabel for DummyEntity {
        fn label(&self) -> &Label {
            &self.label
        }

        fn table() -> &'static str {
            "dummy_table"
        }
    }

    // endregion

    // region: Label::new

    /// Verifies that `Label::new` normalizes raw names to title case and stores the description.
    #[test]
    fn new_formats_name_to_title_case_and_keeps_description() {
        // Arrange
        let raw_name = "my_test-string";
        let description = Some("a description");

        // Act
        let label = Label::new(raw_name, description);

        // Assert
        assert_eq!(label.name, "My Test String");
        assert_eq!(label.description, Some(String::from("a description")));
    }

    /// Verifies that `Label::new` maps an absent optional description field to `None`.
    #[test]
    fn new_with_no_description_stores_none() {
        // Arrange
        let raw_name = "fund_name";

        // Act
        let label = Label::new(raw_name, None);

        // Assert
        assert_eq!(label.description, None);
    }

    /// Verifies that sequential calls to `Label::new` generate distinct, non-nil identifiers.
    #[test]
    fn new_generates_a_non_nil_unique_id_each_call() {
        // Act
        let first = Label::new("foo", None);
        let second = Label::new("foo", None);

        // Assert
        assert_ne!(first.id, Uuid::nil());
        assert_ne!(first.id, second.id);
    }

    // endregion

    // region: Label::fmt (Title Case helper)

    /// Verifies that delimiters like underscores and hyphens are accurately parsed into spaces.
    #[test]
    fn fmt_converts_underscores_and_hyphens_to_title_case() {
        // Arrange
        let input = "my_test-string";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "My Test String");
    }

    /// Verifies that sequential delimiters are consolidated cleanly without throwing empty strings.
    #[test]
    fn fmt_collapses_repeated_delimiters_without_empty_words() {
        // Arrange
        let input = "my__test--string  here";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "My Test String Here");
    }

    /// Verifies that passing strings that are already spaced yields expected capitalization rules.
    #[test]
    fn fmt_handles_already_spaced_input() {
        // Arrange
        let input = "already spaced words";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "Already Spaced Words");
    }

    /// Verifies that empty string inputs return clean, zero-length string instances.
    #[test]
    fn fmt_on_empty_string_returns_empty_string() {
        // Arrange
        let input = "";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "");
    }

    /// Verifies that input sequences consisting solely of delimiters process down to empty strings.
    #[test]
    fn fmt_on_only_delimiters_returns_empty_string() {
        // Arrange
        let input = "___---   ";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "");
    }

    /// Verifies that arbitrary inner-word uppercase styling remains unmodified by parsing logic.
    #[test]
    fn fmt_preserves_existing_uppercase_letters_after_first_char() {
        // Arrange
        let input = "hello WORLD";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "Hello WORLD");
    }

    // endregion

    // region: Label::from_row_offset

    /// Verifies successful translation of database rows containing explicit data payloads.
    #[test]
    fn from_row_offset_maps_id_name_and_description() {
        // Arrange
        let conn = connection_with_label_row("Groceries", Some("Weekly shop"));

        // Act
        let result: Label = conn
            .query_row("SELECT id, name, description FROM items", (), |row| {
                Label::from_row_offset(row, 0)
            })
            .expect("query should succeed");

        // Assert
        assert_eq!(result.name, "Groceries");
        assert_eq!(result.description, Some(String::from("Weekly shop")));
    }

    /// Verifies that database NULL states cleanly convert to rust `None` field designations.
    #[test]
    fn from_row_offset_maps_null_description_as_none() {
        // Arrange
        let conn = connection_with_label_row("Groceries", None);

        // Act
        let result: Label = conn
            .query_row("SELECT id, name, description FROM items", (), |row| {
                Label::from_row_offset(row, 0)
            })
            .expect("query should succeed");

        // Assert
        assert_eq!(result.description, None);
    }

    /// Verifies column mapping functions preserve extraction indexes given dynamic query leading offsets.
    #[test]
    fn from_row_offset_respects_a_nonzero_column_offset() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE items (padding INTEGER, id BLOB, name TEXT, description TEXT);",
            (),
        )
        .expect("failed to create table");
        let id = Uuid::new_v4();
        conn.execute(
            "INSERT INTO items (padding, id, name, description) VALUES (1, ?1, ?2, ?3)",
            rusqlite::params![id, "Offset Test", "desc"],
        )
        .expect("failed to insert row");

        // Act
        let result: Label = conn
            .query_row(
                "SELECT padding, id, name, description FROM items",
                (),
                |row| Label::from_row_offset(row, 1),
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(result.name, "Offset Test");
        assert_eq!(result.description, Some(String::from("desc")));
    }

    // endregion

    // region: Label::from_row_offset_no_desc

    /// Verifies parsing paths configured to bypass descriptions actively exclude text columns.
    #[test]
    fn from_row_offset_no_desc_ignores_description_column() {
        // Arrange
        let conn = connection_with_label_row("Groceries", Some("Weekly shop"));

        // Act
        let result: Label = conn
            .query_row("SELECT id, name FROM items", (), |row| {
                Label::from_row_offset_no_desc(row, 0)
            })
            .expect("query should succeed");

        // Assert
        assert_eq!(result.name, "Groceries");
        assert_eq!(result.description, None);
    }

    // endregion

    // region: HasLabel trait default methods

    /// Verifies default trait accessor mapping behavior routes accurately to internal struct fields.
    #[test]
    fn has_label_name_default_delegates_to_label_name() {
        // Arrange
        let entity = DummyEntity {
            label: Label::new("Test Entity", None),
        };

        // Act
        let name = HasLabel::name(&entity);

        // Assert
        assert_eq!(name, "Test Entity");
    }

    /// Verifies default trait wrapper functions map target matching keys effectively.
    #[test]
    fn has_label_id_default_delegates_to_label_id() {
        // Arrange
        let entity = DummyEntity {
            label: Label::new("Test Entity", None),
        };
        let expected_id = entity.label.id;

        // Act
        let id = HasLabel::id(&entity);

        // Assert
        assert_eq!(id, expected_id);
    }

    /// Verifies trait signatures resolve underlying static identifiers matching internal schemas.
    #[test]
    fn has_label_table_returns_static_table_name() {
        // Act
        let table = DummyEntity::table();

        // Assert
        assert_eq!(table, "dummy_table");
    }

    // endregion
}

// endregion
