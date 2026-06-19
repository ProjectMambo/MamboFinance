use crate::user::{HasLabel, Label, NAME_LIMIT, Printable};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone)]
pub struct Currency {
    pub label: Label,
}

impl Currency {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Self::from_row_offset(row, 0)
    }

    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Currency {
            label: Label::from_row_offset_no_desc(row, offset)?,
        })
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            return write!(f, "{:#}", self.label);
        }

        write!(f, "{}", self.label)
    }
}

impl HasLabel for Currency {
    fn name(&self) -> &str {
        &self.label.name
    }

    fn id(&self) -> Uuid {
        self.label.id
    }

    fn table() -> &'static str {
        "currencies"
    }
}

impl Printable for Currency {
    fn title() -> &'static str {
        "CURRENCY"
    }
    fn headers() -> &'static [&'static str] {
        &["NAME"]
    }
    fn widths() -> &'static [usize] {
        &[NAME_LIMIT]
    }
}

impl PartialEq for Currency {
    fn eq(&self, other: &Self) -> bool {
        self.label.name == other.label.name
    }
}
