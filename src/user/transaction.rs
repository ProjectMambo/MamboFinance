use crate::core::{Amount, Date, Label};
use crate::define_struct;
use crate::user::{Category, Fund, Group};
use std::fmt::{Display, Formatter};

define_struct!(
Transaction has{
    crate::core | label: Label,
    crate::core | amount: Amount,
    crate::core | date: Date,
} with{
    group: Group,
    category: Category,
    fund: Fund,
    link: Option<Label>,
});


impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "--- Transaction ---")?;
        writeln!(
            f,
            "{}\namount: {}\ndate: {}\n\n{}\n\n{}\n\n{}",
            self.label, self.amount, self.date, self.group, self.category, self.fund
        )?;
        match &self.link {
            Some(pair) => write!(f, "link:{}", pair),
            None => write!(f, ""),
        }
    }
}

//wrapper to lock possitive, and two digits
//currency: String, //currency enum? or structs? handle convert
// link: Option<String>, //uuid to another linked transactions
