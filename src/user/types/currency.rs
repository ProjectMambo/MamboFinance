use crate::user::HasLabel;
use crate::user::types::Label;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Currency {
    pub label: Label,
}

impl HasLabel for Currency {
    fn id(&self) -> Uuid {
        self.label.id
    }
}

impl Currency {
    pub fn new(name: &str) -> Arc<Self> {
        let label = Label::new_pooled(name, "CURRENCY");
        Arc::new(Self { label })
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:#}", self.label)
    }
}

impl PartialEq for Currency {
    fn eq(&self, other: &Self) -> bool {
        self.label.name == other.label.name
    }
}
