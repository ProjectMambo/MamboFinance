// Imports from internal user module
use crate::user::{HasLabel, Label, Printable};
use std::fmt::{Display, Formatter};

/// Represents a financial currency asset descriptor used for monetary valuation.
#[derive(Clone)]
pub struct Currency {
    pub label: Label,
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

impl Printable for Currency {
    fn title() -> &'static str {
        "CURRENCY"
    }
}

impl PartialEq for Currency {
    /// Compares two currencies solely by their associated lexical name values.
    fn eq(&self, other: &Self) -> bool {
        self.label.name == other.label.name
    }
}
