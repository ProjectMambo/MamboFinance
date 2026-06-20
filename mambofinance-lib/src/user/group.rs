// Imports from internal user module
use crate::user::{HasLabel, Label, Printable};
use std::fmt::{Display, Formatter};

/// Represents a structural category classification used to organize ledger operations.
#[derive(Clone)]
pub struct Group {
    pub label: Label,
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
    fn label(&self) -> &Label {
        &self.label
    }

    fn table() -> &'static str {
        "groups"
    }
}

impl Printable for Group {
    fn title() -> &'static str {
        "GROUP"
    }
}
