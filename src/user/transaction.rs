use crate::user::types::{Amount, Date, Label};
use crate::user::{Category, Fund, Group};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

pub struct Transaction {
    pub label: Label,
    pub amount: Amount,
    pub date: Date,
    pub group: Arc<Group>,
    pub category: Arc<Category>,
    pub fund: Arc<Fund>,
    pub link: Option<Label>,
}

impl Transaction {
    pub fn new(
        label: Label,
        amount: Amount,
        date: Date,
        group: Arc<Group>,
        category: Arc<Category>,
        fund: Arc<Fund>,
        link: Option<Label>,
    ) -> Self {
        Self {
            label,
            amount,
            date,
            group,
            category,
            fund,
            link,
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "--- Transaction ---")?;
        writeln!(
            f,
            "{}\namount: {}\ndate: {}\ngroup: {:#}\ncategory: {:#}\nfund: {:#}",
            self.label, self.amount, self.date, self.group, self.category, self.fund
        )?;
        match &self.link {
            Some(pair) => write!(f, "link:{:#}", pair),
            None => write!(f, ""),
        }
    }
}
