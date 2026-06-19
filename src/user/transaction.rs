use crate::user::{
    AMOUNT_LIMIT, Amount, Category, DESC_LIMIT, Date, Fund, Group, HasLabel, Label, NAME_LIMIT,
    Printable,
};
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
            "{:+} | {} | {} | {} | {:#} | {}",
            self.label, self.amount, self.date, self.group, self.category, self.fund
        )
    }
}

impl HasLabel for Transaction {
    fn name(&self) -> &str {
        &self.label.name
    }

    fn id(&self) -> Uuid {
        self.label.id
    }

    fn table() -> &'static str {
        "transactions"
    }
}

impl Printable for Transaction {
    fn title() -> &'static str {
        "TRANSACTION"
    }
    fn headers() -> &'static [&'static str] {
        &[
            "NAME", "DESC", "AMOUNT", "DATE", "GROUP", "CATEGORY", "FUND",
        ]
    }
    fn widths() -> &'static [usize] {
        &[
            NAME_LIMIT,
            DESC_LIMIT,
            AMOUNT_LIMIT + 13,
            11,
            NAME_LIMIT,
            NAME_LIMIT,
            NAME_LIMIT,
        ]
    }
}
