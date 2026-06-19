use crate::user::types::Label;
use std::fmt::{Display, Formatter};

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
