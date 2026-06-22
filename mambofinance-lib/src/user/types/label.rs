// Imports from internal user module
use crate::user::{DESC_LIMIT, NAME_LIMIT};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

/// Provides unique identity, naming, and optional textual descriptions for database entities.
#[derive(Clone, Debug)]
pub struct Label {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl Label {
    /// Constructs a new `Label` instance with a randomly generated UUID and a formatted name.
    pub fn new(name: &str, description: Option<&str>) -> Self {
        let id = Uuid::new_v4();
        let des = description.map(String::from);

        Self {
            id,
            name: Label::fmt(name),
            description: des,
        }
    }

    /// Maps a single SQLite row to a full `Label` instance including descriptions using column offsets.
    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Label {
            id: row.get(offset)?,
            name: row.get(offset + 1)?,
            description: row.get(offset + 2)?,
        })
    }

    /// Maps a single SQLite row to a partial `Label` instance, explicitly omitting descriptions.
    pub fn from_row_offset_no_desc(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Label {
            id: row.get(offset)?,
            name: row.get(offset + 1)?,
            description: None,
        })
    }

    /// Sanitizes and converts delimiter-separated strings into a Title Case format.
    ///
    /// e.g., "my_test-string" becomes "My Test String".
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
    /// Formats the entity label output, processing truncation and layout padding via type modifiers.
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        // Alternate layout format (`{:#}`)
        if f.alternate() {
            write!(f, "{}", self.name)?;
            return match &self.description {
                Some(des) => write!(f, " | {}", des),
                None => Ok(()),
            };
        }

        // Plus-sign layout format (`{:+}`)
        let des = f.sign_plus().then_some(match &self.description {
            Some(des) => des.as_str(),
            None => "",
        });

        // Standard layout format (`{}`)
        let mut truncated: String = self.name.chars().take(NAME_LIMIT).collect();
        write!(f, "{:<width$}", truncated, width = NAME_LIMIT)?;

        match des {
            Some(des) => {
                truncated = des.chars().take(DESC_LIMIT).collect();
                write!(f, " | {:<width$}", truncated, width = DESC_LIMIT)
            }
            None => Ok(()),
        }
    }
}

/// Interface indicating an entity can expose a uniform metadata `Label` structure and data origin table.
pub trait HasLabel {
    fn name(&self) -> &str {
        &self.label().name
    }

    fn id(&self) -> Uuid {
        self.label().id
    }

    fn label(&self) -> &Label;
    fn table() -> &'static str;
}

// region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // region: helpers

    // Builds a throwaway in-memory connection with a single row containing
    // id, name, and description columns, used to exercise row-mapping constructors.
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

    // A minimal stand-in implementing HasLabel purely to exercise its default methods.
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

    #[test]
    fn new_with_no_description_stores_none() {
        // Arrange
        let raw_name = "fund_name";

        // Act
        let label = Label::new(raw_name, None);

        // Assert
        assert_eq!(label.description, None);
    }

    #[test]
    fn new_generates_a_non_nil_unique_id_each_call() {
        // Arrange
        // Act
        let first = Label::new("foo", None);
        let second = Label::new("foo", None);

        // Assert
        assert_ne!(first.id, Uuid::nil());
        assert_ne!(first.id, second.id);
    }

    // endregion

    // region: Label::fmt (Title Case helper)

    #[test]
    fn fmt_converts_underscores_and_hyphens_to_title_case() {
        // Arrange
        let input = "my_test-string";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "My Test String");
    }

    #[test]
    fn fmt_collapses_repeated_delimiters_without_empty_words() {
        // Arrange
        let input = "my__test--string  here";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "My Test String Here");
    }

    #[test]
    fn fmt_handles_already_spaced_input() {
        // Arrange
        let input = "already spaced words";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "Already Spaced Words");
    }

    #[test]
    fn fmt_on_empty_string_returns_empty_string() {
        // Arrange
        let input = "";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "");
    }

    #[test]
    fn fmt_on_only_delimiters_returns_empty_string() {
        // Arrange
        let input = "___---   ";

        // Act
        let result = Label::fmt(input);

        // Assert
        assert_eq!(result, "");
    }

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

    // region: Display for Label

    #[test]
    fn display_default_pads_name_to_name_limit_width() {
        // Arrange
        let label = Label::new("ab", None);

        // Act
        let rendered = format!("{}", label);

        // Assert
        assert_eq!(rendered.len(), NAME_LIMIT);
        assert_eq!(rendered, format!("{:<width$}", "Ab", width = NAME_LIMIT));
    }

    #[test]
    fn display_default_truncates_name_longer_than_limit() {
        // Arrange
        let long_name = "a".repeat(NAME_LIMIT + 10);
        let label = Label::new(&long_name, None);

        // Act
        let rendered = format!("{}", label);

        // Assert
        assert_eq!(rendered.chars().count(), NAME_LIMIT);
    }

    #[test]
    fn display_default_omits_description_even_when_present() {
        // Arrange
        let label = Label::new("Groceries", Some("Weekly shop"));

        // Act
        let rendered = format!("{}", label);

        // Assert
        assert!(!rendered.contains("Weekly shop"));
        assert!(!rendered.contains('|'));
    }

    #[test]
    fn display_plus_sign_appends_description_when_present() {
        // Arrange
        let label = Label::new("Groceries", Some("Weekly shop"));

        // Act
        let rendered = format!("{:+}", label);

        // Assert
        assert!(rendered.contains("Weekly shop"));
        assert!(rendered.contains('|'));
    }

    #[test]
    fn display_plus_sign_with_no_description_still_renders_separator() {
        // Arrange
        let label = Label::new("Groceries", None);

        // Act
        let rendered = format!("{:+}", label);

        // Assert
        assert!(rendered.contains('|'));
    }

    #[test]
    fn display_plus_sign_truncates_description_to_desc_limit() {
        // Arrange
        let long_desc = "d".repeat(DESC_LIMIT + 10);
        let label = Label::new("Groceries", Some(&long_desc));

        // Act
        let rendered = format!("{:+}", label);
        let desc_part = rendered.split('|').nth(1).expect("should have a desc part");

        // Assert
        assert_eq!(desc_part.trim_end().chars().count(), DESC_LIMIT);
    }

    #[test]
    fn display_alternate_renders_name_only_without_description() {
        // Arrange
        let label = Label::new("Groceries", None);

        // Act
        let rendered = format!("{:#}", label);

        // Assert
        assert_eq!(rendered, "Groceries");
    }

    #[test]
    fn display_alternate_appends_full_description_when_present() {
        // Arrange
        let label = Label::new("Groceries", Some("Weekly shop"));

        // Act
        let rendered = format!("{:#}", label);

        // Assert
        assert_eq!(rendered, "Groceries | Weekly shop");
    }

    // endregion

    // region: HasLabel trait default methods

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

    #[test]
    fn has_label_table_returns_static_table_name() {
        // Arrange
        // Act
        let table = DummyEntity::table();

        // Assert
        assert_eq!(table, "dummy_table");
    }

    // endregion
}

// endregion
