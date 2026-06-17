use crate::user::types::{HasLabel, Label};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Fund {
    pub label: Label,
}

impl HasLabel for Fund {
    fn id(&self) -> Uuid {
        self.label.id
    }
}

impl Fund {
    pub fn new(name: &str) -> Arc<Self> {
        let label = Label::new_pooled(name, "FUND");
        Arc::new(Self { label })
    }
}

impl Display for Fund {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            return write!(f, "{:#}", self.label);
        }

        writeln!(f, "--- Fund ---")?;
        write!(f, "{}", self.label)
    }
}
