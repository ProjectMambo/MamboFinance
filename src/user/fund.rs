use crate::user::{HasLabel, Label, NAME_LIMIT, Printable};
use std::fmt::{Display, Formatter};
use uuid::Uuid;
#[derive(Clone)]
pub struct Fund {
    pub label: Label,
}

impl Fund {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Self::from_row_offset(row, 0)
    }

    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Fund {
            label: Label::from_row_offset_no_desc(row, offset)?,
        })
    }
}

impl Display for Fund {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl HasLabel for Fund {
    fn name(&self) -> &str {
        &self.label.name
    }

    fn id(&self) -> Uuid {
        self.label.id
    }

    fn table() -> &'static str {
        "funds"
    }
}

impl Printable for Fund {
    fn title() -> &'static str {
        "FUND"
    }
    fn headers() -> &'static [&'static str] {
        &["NAME"]
    }
    fn widths() -> &'static [usize] {
        &[NAME_LIMIT]
    }
}
