use crate::user::{HasLabel, Label, NAME_LIMIT, Printable, VARIANT_LIMIT};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone)]
pub struct Category {
    pub label: Label,
    pub variant: CategoryVariant,
}

impl Category {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Self::from_row_offset(row, 0)
    }

    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Category {
            label: Label::from_row_offset_no_desc(row, offset)?,
            variant: row.get(offset + 2)?,
        })
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            return write!(f, "{}", self.label);
        }

        write!(
            f,
            "{} | {:<width$}",
            self.label,
            format!("{:?}", self.variant),
            width = VARIANT_LIMIT,
        )
    }
}

impl HasLabel for Category {
    fn name(&self) -> &str {
        &self.label.name
    }

    fn id(&self) -> Uuid {
        self.label.id
    }

    fn table() -> &'static str {
        "categories"
    }
}

impl Printable for Category {
    fn title() -> &'static str {
        "CATEGORY"
    }
    fn headers() -> &'static [&'static str] {
        &["NAME", "TYPE"]
    }
    fn widths() -> &'static [usize] {
        &[NAME_LIMIT, VARIANT_LIMIT]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CategoryVariant {
    Single = 0,
    Paired = 1,
}

impl rusqlite::types::FromSql for CategoryVariant {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let int_value = value.as_i64()?;
        match int_value {
            0 => Ok(CategoryVariant::Single),
            1 => Ok(CategoryVariant::Paired),
            _ => Err(rusqlite::types::FromSqlError::OutOfRange(int_value)),
        }
    }
}

impl rusqlite::types::ToSql for CategoryVariant {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(*self as i64))
    }
}
