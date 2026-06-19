use crate::user::{DESC_LIMIT, NAME_LIMIT};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Label {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
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

    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Label {
            id: row.get(offset)?,
            name: row.get(offset + 1)?,
            description: row.get(offset + 2)?,
        })
    }

    pub fn from_row_offset_no_desc(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Label {
            id: row.get(offset)?,
            name: row.get(offset + 1)?,
            description: None,
        })
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.name)?;
            return match &self.description {
                Some(des) => write!(f, " | {}", des),
                None => Ok(()),
            };
        }

        let truncated: String = self.name.chars().take(NAME_LIMIT).collect();
        write!(f, "{:<width$}", truncated, width = NAME_LIMIT)?;

        match &self.description {
            Some(des) => {
                let truncated: String = des.chars().take(NAME_LIMIT).collect();
                write!(f, " | {:<width$}", truncated, width = DESC_LIMIT)
            }
            None => Ok(()),
        }
    }
}
