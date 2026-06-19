use crate::user::{
    Category, Currency, Fund, Group, HasLabel, InputError, Label, Transaction, User, UserError,
};
use std::fmt::Display;
use uuid::Uuid;

pub struct Query<'a, T> {
    pub user: &'a User,
    pub rows: Vec<T>,
}

impl<'a, T: HasLabel> Query<'a, T> {
    pub fn sort_by_name(mut self) -> Self {
        self.rows.sort_by(|a, b| a.name().cmp(b.name()));
        self
    }
}

impl<'a> Query<'a, Category> {
    pub fn sort_by_type(mut self) -> Self {
        self.rows.sort_by_key(|c| c.variant);
        self
    }
}

impl<'a> Query<'a, Transaction> {
    pub fn sort_by_date(mut self) -> Self {
        self.rows
            .sort_by_key(|t| (t.date.year, t.date.month, t.date.day));
        self
    }

    pub fn sort_by_amount(mut self) -> Self {
        self.rows.sort_by_key(|t| t.amount.value);
        self
    }

    pub fn sort_by_currency(mut self) -> Self {
        self.rows
            .sort_by(|a, b| a.amount.currency.name().cmp(b.amount.currency.name()));
        self
    }

    pub fn filter_group(mut self, group: &str) -> Self {
        self.rows
            .retain(|t| t.group.label.name == Label::fmt(group));
        self
    }

    pub fn filter_fund(mut self, fund: &str) -> Self {
        self.rows.retain(|t| t.fund.label.name == Label::fmt(fund));
        self
    }

    pub fn filter_category(mut self, category: &str) -> Self {
        self.rows
            .retain(|t| t.category.label.name == Label::fmt(category));
        self
    }

    pub fn filter_currency(mut self, currency: &str) -> Self {
        self.rows
            .retain(|t| t.amount.currency.label.name == Label::fmt(currency));
        self
    }
}

pub trait Printable {
    fn title() -> &'static str;
    fn headers() -> &'static [&'static str];
    fn widths() -> &'static [usize];
}

impl<'a, T: Display + Printable> Query<'a, T> {
    pub fn print(self) -> Self {
        self.user
            .print_table(T::title(), T::headers(), T::widths(), &self.rows);
        self
    }
}

impl<'a, T: HasLabel> Query<'a, T> {
    fn delete_by_id(mut self, id: Uuid, also: Option<Uuid>) -> Result<Self, UserError> {
        match also {
            Some(link) => {
                self.user
                    .conn
                    .execute(
                        &format!("DELETE FROM {} WHERE id = ?1 OR id = ?2", T::table()),
                        rusqlite::params![id, link],
                    )
                    .map_err(UserError::SQL)?;
                self.rows.retain(|r| r.id() != id && r.id() != link);
            }
            None => {
                self.user
                    .conn
                    .execute(&format!("DELETE FROM {} WHERE id = ?1", T::table()), [id])
                    .map_err(UserError::SQL)?;
                self.rows.retain(|r| r.id() != id);
            }
        }
        Ok(self)
    }
}

impl<'a> Query<'a, Group> {
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        let id = self
            .rows
            .get(no - 1)
            .map(|r| r.id())
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))?;
        self.delete_by_id(id, None)
    }
}

impl<'a> Query<'a, Fund> {
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        let id = self
            .rows
            .get(no - 1)
            .map(|r| r.id())
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))?;
        self.delete_by_id(id, None)
    }
}

impl<'a> Query<'a, Category> {
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        let id = self
            .rows
            .get(no - 1)
            .map(|r| r.id())
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))?;
        self.delete_by_id(id, None)
    }
}

impl<'a> Query<'a, Currency> {
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        let id = self
            .rows
            .get(no - 1)
            .map(|r| r.id())
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))?;
        self.delete_by_id(id, None)
    }
}

impl<'a> Query<'a, Transaction> {
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        let row = self
            .rows
            .get(no - 1)
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))?;
        let id = row.label.id;
        let link = row.link;
        self.delete_by_id(id, link)
    }
}
