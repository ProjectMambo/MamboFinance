use crate::user::types::{HasLabel, Label};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Group {
    pub label: Label,
}

impl HasLabel for Group {
    fn id(&self) -> Uuid {
        self.label.id
    }
}

impl Group {
    pub fn new(name: &str) -> Arc<Self> {
        let label = Label::new_pooled(name, "GROUP");
        Arc::new(Self { label })
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            return write!(f, "{:#}", self.label);
        }

        writeln!(f, "--- Group ---")?;
        write!(f, "{}", self.label)
    }
}
