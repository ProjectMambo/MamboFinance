use crate::{core::Label, define_struct};
use std::fmt::{Display, Formatter};

define_struct!(
Fund has{
    crate::core | label: Label,
});

impl Display for Fund {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "--- Fund ---")?;
        write!(f, "{}", self.label)
    }
}
