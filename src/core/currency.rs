use crate::define_struct;
use std::fmt::{Display, Formatter};

define_struct!(
Currency with {
    name: String,
});

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Currency {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
