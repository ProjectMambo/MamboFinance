use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Label {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

pub trait HasLabel {
    fn id(&self) -> Uuid;
}

impl Label {
    pub fn new(name: &str, description: Option<&str>) -> Self {
        let id = Uuid::new_v4();
        let des = description.map(String::from);

        Self {
            id,
            name: String::from(name),
            description: des,
        }
    }

    pub fn new_pooled(name: &str, description: &str) -> Self {
        //FIXME: change hardcode
        let namespace = Uuid::parse_str("67c87747-17e2-4bc4-9d58-9a996924b107").unwrap();
        let namedes = format!("{}-{}", name, description);

        let id = Uuid::new_v5(&namespace, namedes.as_bytes());
        Self {
            id,
            name: String::from(name),
            description: Some(String::from(description)),
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            return write!(f, "{}", self.name);
        }

        if f.sign_plus() {
            write!(f, "id: {}\nname: {}", self.id, self.name)?;
            return match &self.description {
                Some(des) => write!(f, "\ndescription: {}", des),
                None => Ok(()),
            };
        }

        write!(f, "name: {}", self.name)?;
        match &self.description {
            Some(des) => write!(f, "\ndescription: {}", des),
            None => Ok(()),
        }
    }
}
