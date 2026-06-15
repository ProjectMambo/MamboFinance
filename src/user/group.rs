use crate::{core::Label, define_struct};
use std::fmt::{Display, Formatter};

define_struct!(
Group has{
    crate::core | label: Label,
});

impl Display for Group {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "--- Group ---")?;
        write!(f, "{}", self.label)
    }
}
