use crate::user::types::{Amount, Date, Label};
use crate::user::{Category, Fund, Group};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Transaction {
    pub label: Label,
    pub amount: Amount,
    pub date: Date,
    pub group: Group,
    pub category: Category,
    pub fund: Fund,
    pub link: Option<Uuid>,
}

impl Transaction {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Transaction {
            label: Label::from_row_offset(row, 0)?,        // 0,1,2
            amount: Amount::from_row_offset(row, 3)?,      // 3,4,5
            date: Date::from_row_offset(row, 6)?,          // 6,7,8
            group: Group::from_row_offset(row, 9)?,        // 9,10
            category: Category::from_row_offset(row, 11)?, // 11,12,13
            fund: Fund::from_row_offset(row, 14)?,         // 14,15
            link: row.get(16)?,
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {:#} | {}",
            self.label, self.amount, self.date, self.group, self.category, self.fund
        )
        // match &self.link {
        //     Some(pair) => write!(f, " {}", pair),
        //     None => write!(f, ""),
        // }
    }
}
