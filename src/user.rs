mod category;
mod fund;
mod group;
mod transaction;
mod types;

use category::*;
use fund::*;
use group::*;
use transaction::*;
use types::*;

use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

pub struct User {
    label: Label,
    transactions: Pool<Transaction>,
    groups: Pool<Group>,
    categories: Pool<Category>,
    funds: Pool<Fund>,
    currencies: Pool<Currency>,
}

#[derive(Error, Debug)]
pub enum InputError {
    #[error(
        "{0} hasn't been initialized, use add_{1} to initialize {1} before adding transaction."
    )]
    UninitInput(String, String),
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error(transparent)]
    Input(#[from] InputError),

    #[error(transparent)]
    Date(#[from] DateError),
}

impl User {
    pub fn new(name: &str) -> Self {
        User {
            label: Label::new_pooled(name, "USER"),
            transactions: Pool::new(),
            groups: Pool::new(),
            categories: Pool::new(),
            funds: Pool::new(),
            currencies: Pool::new(),
        }
    }

    pub fn add_transaction(
        &self,
        name: &str,
        description: Option<&str>,
        (amount, currency): (i64, &str),
        (day, month, year): (u8, u8, u16),
        group: &str,
        category: &str,
        fund: &str,
    ) -> Result<(), TransactionError> {//FIXME: check category type
        let currency = self.get_currency(currency)?;
        let group = self.get_group(group)?;
        let category = self.get_category(category)?;
        let fund = self.get_fund(fund)?;

        let label = Label::new(name, description);
        let amount = Amount::new(amount, currency);
        let date = Date::new(day, month, year)?;

        let t = Transaction::new(label, amount, date, group, category, fund, None);
        println!("\n{t}");
        Ok(())
    }

    pub fn add_paired_transaction(
        &self,
        name: &str,
        description: Option<&str>,
        (amount, currency): (i64, &str),
        (day, month, year): (u8, u8, u16),
        group: &str,
        category: &str,
        source_fund: &str,
        target_fund: &str,
    ) -> Result<(), TransactionError> {
        let currency = self.get_currency(currency)?;
        let group = self.get_group(group)?;
        let category = self.get_category(category)?;
        let source_fund = self.get_fund(source_fund)?;
        let target_fund = self.get_fund(target_fund)?;

        let label = Label::new(name, description);
        let amount = Amount::new(amount, currency);
        let date = Date::new(day, month, year)?;

        let t = Transaction::new(
            label.clone(),
            amount.reverse(),
            date.clone(),
            group.clone(),
            category.clone(),
            target_fund,
            Some(label.clone()),
        );
        let s = Transaction::new(
            label.clone(),
            amount,
            date,
            group,
            category,
            source_fund,
            Some(label),
        );
        println!("\n{s}");
        println!("\n{t}");
        Ok(())
    }

    pub fn unwrap_result(result: Result<(), TransactionError>) {
        match result {
            Ok(_) => {}
            Err(e) => println!("Error: {}", e),
        }
    }

    pub fn add_group(&self, name: &str) {
        User::add_pooled(&self.groups, name, "GROUP", || Group::new(name));
    }
    fn get_group(&self, name: &str) -> Result<Arc<Group>, InputError> {
        User::get_pooled(&self.groups, name, "GROUP", "group")
    }

    pub fn add_category(&self, name: &str) {
        User::add_pooled(&self.categories, name, "CATEGORY", || {
            Category::new(name, None)
        });
    }
    pub fn add_paired_category(&self, name: &str) {
        User::add_pooled(&self.categories, name, "CATEGORY", || {
            Category::new(name, Some(CategoryVariant::Paired))
        });
    }
    fn get_category(&self, name: &str) -> Result<Arc<Category>, InputError> {
        User::get_pooled(&self.categories, name, "CATEGORY", "category")
    }

    pub fn add_fund(&self, name: &str) {
        User::add_pooled(&self.funds, name, "FUND", || Fund::new(name));
    }
    fn get_fund(&self, name: &str) -> Result<Arc<Fund>, InputError> {
        User::get_pooled(&self.funds, name, "FUND", "fund")
    }

    pub fn add_currency(&self, name: &str) {
        User::add_pooled(&self.currencies, name, "CURRENCY", || Currency::new(name));
    }
    fn get_currency(&self, name: &str) -> Result<Arc<Currency>, InputError> {
        User::get_pooled(&self.currencies, name, "CURRENCY", "currency")
    }

    fn add_pooled<T, F>(pool: &Pool<T>, name: &str, type_tag: &str, make: F)
    where
        T: HasLabel,
        F: FnOnce() -> Arc<T>,
    {
        let label = Label::new_pooled(name, type_tag);
        if pool.get(&label).is_none() {
            pool.insert(make());
            return println!("Added new {} called {}!", type_tag, name);
        }
        println!("{} already exist for {}!", name, type_tag)
    }

    fn get_pooled<T>(
        pool: &Pool<T>,
        name: &str,
        type_tag: &str,
        kind: &str,
    ) -> Result<Arc<T>, InputError>
    where
        T: HasLabel,
    {
        let label = Label::new_pooled(name, type_tag);
        pool.get(&label)
            .ok_or_else(|| InputError::UninitInput(String::from(name), String::from(kind)))
    }
}
