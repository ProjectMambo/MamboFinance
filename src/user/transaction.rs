// Imports from internal user module
use crate::user::{
    AMOUNT_LIMIT, Amount, Category, DESC_LIMIT, Date, Fund, Group, HasLabel, Label, NAME_LIMIT,
    Printable,
};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

/// Represents a financial transaction entry within the accounting system.
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
    /// Maps a single SQLite row to a `Transaction` instance using explicit column offsets.
    ///
    /// The expected layout of the query results must match the comment annotations below.
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Transaction {
            label: Label::from_row_offset(row, 0)?, // Columns: 0 (id), 1 (name), 2 (description)
            amount: Amount::from_row_offset(row, 3)?, // Columns: 3 (value), 4 (currency id), 5 (currency name)
            date: Date::from_row_offset(row, 6)?,     // Columns: 6 (day), 7 (month), 8 (year)
            group: Group::from_row_offset(row, 9)?,   // Columns: 9 (id), 10 (name)
            category: Category::from_row_offset(row, 11)?, // Columns: 11 (id), 12 (name), 13 (variant)
            fund: Fund::from_row_offset(row, 14)?,         // Columns: 14 (id), 15 (name)
            link: row.get(16)?,                            // Column:  16 (linked transaction id)
        })
    }
}

impl Display for Transaction {
    /// Formats the transaction data as a single-line pipe-separated string.
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
            AMOUNT_LIMIT + 13, // Standard amount character allowance with spacing padded for currency codes
            11,                // Formatted date string padding allocation (DD-MM-YYYY)
            NAME_LIMIT,
            NAME_LIMIT,
            NAME_LIMIT,
        ]
    }
}
