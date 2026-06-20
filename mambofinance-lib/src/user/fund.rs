// Imports from internal user module
use crate::user::{HasLabel, Label, Printable};
use std::fmt::{Display, Formatter};

/// Represents an asset storage account or location (e.g., Cash, Bank, Savings) within the system.
#[derive(Clone)]
pub struct Fund {
    pub label: Label,
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
    fn label(&self) -> &Label {
        &self.label
    }

    fn table() -> &'static str {
        "funds"
    }
}

impl Printable for Fund {
    fn title() -> &'static str {
        "FUND"
    }
}
