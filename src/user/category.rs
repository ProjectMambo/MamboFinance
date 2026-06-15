use crate::{core::Label, define_struct};
use std::fmt::{Display, Formatter};

define_struct!(
Category has{
    crate::core | label: Label,
} with{
    variant: CategoryVariant,
});

#[derive(Clone, Debug)]
pub enum CategoryVariant {
    Single,
    Paired,
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "--- Cagtegory ---")?;
        write!(f, "{}\ntype: {:?}", self.label, self.variant)
    }
}
