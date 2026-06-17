use crate::user::HasLabel;
use crate::user::types::Label;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Category {
    pub label: Label,
    pub variant: CategoryVariant,
}

impl HasLabel for Category {
    fn id(&self) -> Uuid {
        self.label.id
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CategoryVariant {
    Single,
    Paired,
}

impl Category {
    pub fn new(name: &str, variant: Option<CategoryVariant>) -> Arc<Self> {
        let label = Label::new_pooled(name, "CATEGORY");
        let category_variant = match &variant {
            Some(var) => *var,

            None => CategoryVariant::Single,
        };
        Arc::new(Self {
            label,
            variant: category_variant,
        })
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            return write!(f, "{:#}", self.label);
        }

        writeln!(f, "--- Cagtegory ---")?;
        write!(f, "{}\ntype: {:?}", self.label, self.variant)
    }
}
