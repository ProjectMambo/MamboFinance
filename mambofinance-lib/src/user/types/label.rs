// Imports from internal user module
use crate::user::{DESC_LIMIT, NAME_LIMIT};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

/// Provides unique identity, naming, and optional textual descriptions for database entities.
#[derive(Clone)]
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
