use crate::user::{HasLabel, Label, NAME_LIMIT, Printable};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone)]
pub struct Group {
    pub label: Label,
}

impl Group {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Self::from_row_offset(row, 0)
    }

    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Group {
            label: Label::from_row_offset_no_desc(row, offset)?,
        })
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl HasLabel for Group {
    fn name(&self) -> &str {
        &self.label.name
    }

    fn id(&self) -> Uuid {
        self.label.id
    }

    fn table() -> &'static str {
        "groups"
    }
}

impl Printable for Group {
    fn title() -> &'static str {
        "GROUP"
    }
    fn headers() -> &'static [&'static str] {
        &["NAME"]
    }
    fn widths() -> &'static [usize] {
        &[NAME_LIMIT]
    }
}
