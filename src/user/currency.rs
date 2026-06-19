use crate::user::types::Label;
use std::fmt::{Display, Formatter};

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

impl PartialEq for Currency {
    fn eq(&self, other: &Self) -> bool {
        self.label.name == other.label.name
    }
}
