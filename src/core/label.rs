use crate::define_struct;
use std::fmt::{Display, Formatter};

define_struct!(
Label with{
    id: i32,
    name: String,
    description: Option<String>,
});

impl Display for Label {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "id: {}\nname: {}", self.id, self.name)?;
        match &self.description {
            Some(des) => write!(f, "\ndescription: {}", des),
            None => Ok(()),
        }
    }
}
