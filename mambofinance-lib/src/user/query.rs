// Imports from internal user module
use crate::user::{
    Category, Currency, Date, Fund, Group, HasLabel,
    InputError::{self, ExistingItem},
    Label, NAME_LIMIT, Transaction, User, UserError,
    category::CategoryVariant,
};
use std::collections::HashMap;
use uuid::Uuid;

/// A builder-like query wrapper for sorting, filtering, displaying, and removing ledger records.
pub struct Query<'a, T> {
    pub user: &'a User,
    pub rows: Vec<T>,
}

impl<'a, T: HasLabel> Query<'a, T> {
    /// Sorts the internal dataset alphabetically by name using lexical sorting order.
    pub fn sort_by_name(mut self) -> Self {
        self.rows.sort_by(|a, b| a.name().cmp(b.name()));
        self
    }
}

impl<'a> Query<'a, Category> {
    /// Sorts categories sequentially by their operational structure type (Single vs Paired).
    pub fn sort_by_type(mut self) -> Self {
        self.rows.sort_by_key(|c| c.variant);
        self
    }
}

impl<'a> Query<'a, Transaction> {
    /// Sorts transactions chronologically from oldest to newest.
    pub fn sort_by_date(mut self) -> Self {
        self.rows.sort_by_key(|t| t.date);
        self
    }

    /// Sorts transactions in ascending order by their numeric raw value.
    pub fn sort_by_amount(mut self) -> Self {
        self.rows.sort_by_key(|t| t.amount.value);
        self
    }

    pub fn sort_by_abs_amount(mut self) -> Self {
        self.rows.sort_by_key(|t| t.amount.value.abs());
        self
    }

    pub fn sort_by_flow(mut self) -> Self {
        self.rows.sort_by_key(|t| t.amount.value < 0);
        self
    }

    /// Sorts transactions alphabetically by the lexical name of their respective asset currencies.
    pub fn sort_by_currency(mut self) -> Self {
        self.rows
            .sort_by(|a, b| a.amount.currency.name().cmp(b.amount.currency.name()));
        self
    }

    /// Filters the internal collection to only contain transactions belonging to the given group.
    pub fn filter_group(self, group: &str) -> Self {
        self.filter(|t| t.group.label.name == Label::fmt(group))
    }

    /// Filters the internal collection to only contain transactions tied to the given asset fund/account.
    pub fn filter_fund(self, fund: &str) -> Self {
        self.filter(|t| t.fund.label.name == Label::fmt(fund))
    }

    /// Filters the internal collection to only contain transactions matching the given category.
    pub fn filter_category(self, category: &str) -> Self {
        self.filter(|t| t.category.label.name == Label::fmt(category))
    }

    /// Filters the internal collection to only contain transactions using the specified currency asset type.
    pub fn filter_currency(self, currency: &str) -> Self {
        self.filter(|t| t.amount.currency.label.name == Label::fmt(currency))
    }
}

/// Abstract representation of printable terminal tables providing structural spacing details.
pub trait Printable {
    fn title() -> &'static str;
    fn headers() -> &'static [&'static str] {
        &["NAME"]
    }
    fn widths() -> &'static [usize] {
        &[NAME_LIMIT]
    }
}

impl<'a, T: HasLabel> Query<'a, T> {
    fn get_by_index(&self, no: usize) -> Result<Uuid, UserError> {
        if no == 0 {
            return Err(UserError::Input(InputError::InvalidIndex(no)));
        }

        self.rows
            .get(no - 1)
            .map(|r| r.id())
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))
    }

    /// Base internal helper to drop specific database primary keys and matching memory rows.
    ///
    /// Cleans up paired cross-references automatically if a link identifier is provided.
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

    fn edit_unique_by_id(
        self,
        id: Uuid,
        name: &str,
        edit: impl FnOnce(&rusqlite::Connection, Uuid) -> rusqlite::Result<()>,
    ) -> Result<Self, UserError> {
        let existing = self.user.get_from_table::<T>(name);

        match existing {
            Ok(existing_id) if existing_id != id => Err(UserError::Input(ExistingItem(
                String::from(name),
                String::from(T::table()),
            ))),
            Ok(_) => {
                edit(&self.user.conn, id).map_err(UserError::SQL)?;
                Ok(self)
            }
            Err(UserError::SQL(rusqlite::Error::QueryReturnedNoRows)) => {
                edit(&self.user.conn, id).map_err(UserError::SQL)?;
                Ok(self)
            }
            Err(e) => Err(e),
        }
    }

    fn edit_by_id(
        self,
        id: Uuid,
        edit: impl FnOnce(&rusqlite::Connection, Uuid) -> rusqlite::Result<()>,
    ) -> Result<Self, UserError> {
        edit(&self.user.conn, id).map_err(UserError::SQL)?;
        Ok(self)
    }
}

impl<'a> Query<'a, Group> {
    pub fn print(self) -> Self {
        self.user.print_table(
            Group::title(),
            Group::headers(),
            Group::widths(),
            &self.rows,
        );
        self
    }

    /// Removes a group from the database using its visible 1-indexed table position.
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        self.delete_by_id(id, None)
    }

    pub fn edit_name(self, no: usize, new_name: &str) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        let formatted_name = Label::fmt(new_name);

        self.edit_unique_by_id(id, &formatted_name, |conn, id| {
            conn.execute(
                "UPDATE groups SET name = ?1 WHERE id = ?2",
                rusqlite::params![formatted_name.clone(), id],
            )
            .map(|_| ())
        })
    }
}

impl<'a> Query<'a, Fund> {
    pub fn print(self) -> Self {
        self.user
            .print_table(Fund::title(), Fund::headers(), Fund::widths(), &self.rows);
        self
    }

    /// Removes a fund entry from the database using its visible 1-indexed table position.
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        self.delete_by_id(id, None)
    }

    pub fn edit_name(self, no: usize, new_name: &str) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        let formatted_name = Label::fmt(new_name);

        self.edit_unique_by_id(id, &formatted_name, |conn, id| {
            conn.execute(
                "UPDATE funds SET name = ?1 WHERE id = ?2",
                rusqlite::params![formatted_name.clone(), id],
            )
            .map(|_| ())
        })
    }
}

impl<'a> Query<'a, Category> {
    pub fn print(self) -> Self {
        self.user.print_table(
            Category::title(),
            Category::headers(),
            Category::widths(),
            &self.rows,
        );
        self
    }

    /// Removes a category from the database using its visible 1-indexed table position.
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        self.delete_by_id(id, None)
    }

    pub fn edit_name(self, no: usize, new_name: &str) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        let formatted_name = Label::fmt(new_name);

        self.edit_unique_by_id(id, &formatted_name, |conn, id| {
            conn.execute(
                "UPDATE categories SET name = ?1 WHERE id = ?2",
                rusqlite::params![formatted_name.clone(), id],
            )
            .map(|_| ())
        })
    }

    pub fn edit_variant(self, no: usize, new_variant: CategoryVariant) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;

        let in_use: i64 = self
            .user
            .conn
            .query_row(
                "SELECT COUNT(*) FROM transactions WHERE category_id = ?1",
                [id],
                |row| row.get(0),
            )
            .map_err(UserError::SQL)?;

        if in_use > 0 {
            return Err(UserError::Input(InputError::CategoryInUse(in_use as usize)));
        }

        self.edit_by_id(id, |conn, id| {
            conn.execute(
                "UPDATE categories SET variant = ?1 WHERE id = ?2",
                rusqlite::params![new_variant, id],
            )
            .map(|_| ())
        })
    }

    pub fn force_edit_variant(
        self,
        no: usize,
        new_variant: CategoryVariant,
    ) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;

        self.user
            .conn
            .execute(
                "UPDATE transactions SET link_id = NULL WHERE category_id = ?1",
                [id],
            )
            .map_err(UserError::SQL)?;

        self.edit_by_id(id, |conn, id| {
            conn.execute(
                "UPDATE categories SET variant = ?1 WHERE id = ?2",
                rusqlite::params![new_variant, id],
            )
            .map(|_| ())
        })
    }
}

impl<'a> Query<'a, Currency> {
    pub fn print(self) -> Self {
        self.user.print_table(
            Currency::title(),
            Currency::headers(),
            Currency::widths(),
            &self.rows,
        );
        self
    }

    /// Removes an asset currency from the database using its visible 1-indexed table position.
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        self.delete_by_id(id, None)
    }

    pub fn edit_name(self, no: usize, new_name: &str) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        let formatted_name = Label::fmt(new_name);

        self.edit_unique_by_id(id, &formatted_name, |conn, id| {
            conn.execute(
                "UPDATE currencies SET name = ?1 WHERE id = ?2",
                rusqlite::params![formatted_name.clone(), id],
            )
            .map(|_| ())
        })
    }
}

impl<'a> Query<'a, Transaction> {
    pub fn print(self) -> Self {
        let id_to_index: HashMap<Uuid, usize> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, t)| (t.label.id, i + 1))
            .collect();

        let link_labels: Vec<String> = self
            .rows
            .iter()
            .map(|t| match t.link {
                None => "-".to_string(),
                Some(link_id) => id_to_index
                    .get(&link_id)
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "OOR".to_string()),
            })
            .collect();

        self.user.print_table_with_link(
            Transaction::title(),
            Transaction::headers(), // 7 headers, NOT including LINK
            Transaction::widths(),  // 7 widths, NOT including LINK
            &self.rows,
            "LINK",
            &link_labels,
        );
        self
    }

    /// Removes a single transaction or a double-entry pair from the database using its visible table position.
    pub fn delete(self, no: usize) -> Result<Self, UserError> {
        if no == 0 {
            return Err(UserError::Input(InputError::InvalidIndex(no)));
        }
        let row = self
            .rows
            .get(no - 1)
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))?;
        let id = row.label.id;
        let link = row.link;
        self.delete_by_id(id, link)
    }

    fn edit_shared_field(
        &self,
        no: usize,
        column: &str,
        value: impl rusqlite::ToSql,
    ) -> Result<(), UserError> {
        if no == 0 {
            return Err(UserError::Input(InputError::InvalidIndex(no)));
        }

        let row = self
            .rows
            .get(no - 1)
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))?;

        let id = row.label.id;
        match row.link {
            Some(other_id) => self.user.conn.execute(
                &format!("UPDATE transactions SET {column} = ?1 WHERE id = ?2 OR id = ?3"),
                rusqlite::params![value, id, other_id],
            ),
            None => self.user.conn.execute(
                &format!("UPDATE transactions SET {column} = ?1 WHERE id = ?2"),
                rusqlite::params![value, id],
            ),
        }
        .map_err(UserError::SQL)?;
        Ok(())
    }

    pub fn edit_name(self, no: usize, new_name: &str) -> Result<Self, UserError> {
        let formatted = Label::fmt(new_name);
        self.edit_shared_field(no, "name", formatted)?;
        Ok(self)
    }

    pub fn edit_group(self, no: usize, new_group: &str) -> Result<Self, UserError> {
        let group_id = self.user.get_group(&Label::fmt(new_group))?;
        self.edit_shared_field(no, "group_id", group_id)?;
        Ok(self)
    }

    pub fn edit_date(self, no: usize, day: u8, month: u8, year: u16) -> Result<Self, UserError> {
        if no == 0 {
            return Err(UserError::Input(InputError::InvalidIndex(no)));
        }

        let date = Date::new(day, month, year)?;
        let row = self
            .rows
            .get(no - 1)
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))?;
        let id = row.label.id;
        match row.link {
            Some(other_id) => self.user.conn.execute(
                "UPDATE transactions SET day=?1, month=?2, year=?3 WHERE id = ?4 OR id = ?5",
                rusqlite::params![date.day, date.month, date.year, id, other_id],
            ),
            None => self.user.conn.execute(
                "UPDATE transactions SET day=?1, month=?2, year=?3 WHERE id = ?4",
                rusqlite::params![date.day, date.month, date.year, id],
            ),
        }
        .map_err(UserError::SQL)?;
        Ok(self)
    }

    pub fn edit_category(self, no: usize, new_category: &str) -> Result<Self, UserError> {
        if no == 0 {
            return Err(UserError::Input(InputError::InvalidIndex(no)));
        }
        let row = self
            .rows
            .get(no - 1)
            .ok_or(UserError::Input(InputError::InvalidIndex(no)))?;

        let id = row.label.id;
        let link = row.link;
        let is_paired = link.is_some();

        let formatted = Label::fmt(new_category);
        let new_category_id = self.user.get_category(&formatted)?;

        let required_variant = if is_paired {
            CategoryVariant::Paired
        } else {
            CategoryVariant::Single
        };
        self.user
            .check_category_variant(new_category_id, required_variant)?;

        match link {
            Some(other_id) => self.user.conn.execute(
                "UPDATE transactions SET category_id = ?1 WHERE id = ?2 OR id = ?3",
                rusqlite::params![new_category_id, id, other_id],
            ),
            None => self.user.conn.execute(
                "UPDATE transactions SET category_id = ?1 WHERE id = ?2",
                rusqlite::params![new_category_id, id],
            ),
        }
        .map_err(UserError::SQL)?;

        Ok(self)
    }

    pub fn edit_fund(self, no: usize, new_fund: &str) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        let formatted_name = Label::fmt(new_fund);
        let fund_id = self.user.get_fund(&formatted_name)?;

        self.edit_by_id(id, |conn, id| {
            conn.execute(
                "UPDATE transactions SET fund_id = ?1 WHERE id = ?2",
                rusqlite::params![fund_id, id],
            )
            .map(|_| ())
        })
    }

    pub fn edit_amount(self, no: usize, amount: i64) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        self.edit_by_id(id, |conn, id| {
            conn.execute(
                "UPDATE transactions SET amount = ?1 WHERE id = ?2",
                rusqlite::params![amount, id],
            )
            .map(|_| ())
        })
    }

    pub fn edit_currency(self, no: usize, new_currency: &str) -> Result<Self, UserError> {
        let id = self.get_by_index(no)?;
        let formatted_name = Label::fmt(new_currency);
        let currency_id = self.user.get_currency(&formatted_name)?;

        self.edit_by_id(id, |conn, id| {
            conn.execute(
                "UPDATE transactions SET currency_id = ?1 WHERE id = ?2",
                rusqlite::params![currency_id, id],
            )
            .map(|_| ())
        })
    }
}

// ============================================================
// Only covers PRIVATE methods that tests/ cannot reach:
// get_by_index, delete_by_id, edit_unique_by_id, edit_by_id,
// and Transaction's private edit_shared_field.
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::User;

    fn setup() -> User {
        User::new_in_memory("test").unwrap()
    }

    // ===== get_by_index (private) =====

    #[test]
    fn get_by_index_returns_correct_uuid() {
        let user = setup();
        user.add_group("Food").unwrap();
        let groups = user.ls_group().unwrap();
        let expected_id = groups[0].label.id;

        let query = user.groups().unwrap();
        assert_eq!(query.get_by_index(1).unwrap(), expected_id);
    }

    #[test]
    fn get_by_index_out_of_range_errors() {
        let user = setup();
        user.add_group("Food").unwrap();
        let query = user.groups().unwrap();
        assert!(query.get_by_index(99).is_err());
    }

    #[test]
    fn get_by_index_zero_errors() {
        // no - 1 underflow guard: index 0 should error, not panic
        let user = setup();
        user.add_group("Food").unwrap();
        let query = user.groups().unwrap();
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| query.get_by_index(0)));
        match result {
            Ok(r) => assert!(r.is_err(), "index 0 should return an error, not Ok"),
            Err(_) => panic!(
                "get_by_index(0) panicked instead of returning an error (likely usize underflow on `no - 1`)"
            ),
        }
    }

    // ===== delete_by_id (private) =====

    #[test]
    fn delete_by_id_removes_single_row() {
        let user = setup();
        user.add_group("Food").unwrap();
        let id = user.get_group("Food").unwrap();

        let query = user.groups().unwrap();
        let query = query.delete_by_id(id, None).unwrap();
        assert_eq!(query.rows.len(), 0);
        assert_eq!(user.ls_group().unwrap().len(), 0);
    }

    #[test]
    fn delete_by_id_removes_linked_pair() {
        let user = setup();
        user.add_currency("MYR")
            .unwrap()
            .add_group("Transfer")
            .unwrap()
            .add_paired_category("Internal")
            .unwrap()
            .add_fund("Checking")
            .unwrap()
            .add_fund("Savings")
            .unwrap();
        user.add_paired_transaction(
            "Move",
            None,
            (50000, "MYR"),
            (50000, "MYR"),
            (1, 6, 2026),
            "Transfer",
            "Internal",
            "Checking",
            "Savings",
        )
        .unwrap();

        let transactions = user.ls_transaction().unwrap();
        let id = transactions[0].label.id;
        let link = transactions[0].link;

        let query = user.transactions().unwrap();
        let query = query.delete_by_id(id, link).unwrap();
        assert_eq!(query.rows.len(), 0);
        assert_eq!(user.ls_transaction().unwrap().len(), 0);
    }

    // ===== edit_unique_by_id (private) =====

    #[test]
    fn edit_unique_by_id_allows_renaming_to_same_name() {
        let user = setup();
        user.add_group("Food").unwrap();
        let id = user.get_group("Food").unwrap();

        let query = user.groups().unwrap();
        let result = query.edit_unique_by_id(id, "Food", |conn, id| {
            conn.execute(
                "UPDATE groups SET name = ?1 WHERE id = ?2",
                rusqlite::params!["Food", id],
            )
            .map(|_| ())
        });
        assert!(
            result.is_ok(),
            "renaming a row to its own current name should not be treated as a collision"
        );
    }

    #[test]
    fn edit_unique_by_id_blocks_collision_with_other_row() {
        let user = setup();
        user.add_group("Food").unwrap();
        user.add_group("Housing").unwrap();
        let food_id = user.get_group("Food").unwrap();

        let query = user.groups().unwrap();
        let result = query.edit_unique_by_id(food_id, "Housing", |conn, id| {
            conn.execute(
                "UPDATE groups SET name = ?1 WHERE id = ?2",
                rusqlite::params!["Housing", id],
            )
            .map(|_| ())
        });
        assert!(result.is_err());
    }

    #[test]
    fn edit_unique_by_id_allows_new_unused_name() {
        let user = setup();
        user.add_group("Food").unwrap();
        let id = user.get_group("Food").unwrap();

        let query = user.groups().unwrap();
        let result = query.edit_unique_by_id(id, "Groceries", |conn, id| {
            conn.execute(
                "UPDATE groups SET name = ?1 WHERE id = ?2",
                rusqlite::params!["Groceries", id],
            )
            .map(|_| ())
        });
        assert!(result.is_ok());
        assert_eq!(user.ls_group().unwrap()[0].label.name, "Groceries");
    }

    // ===== edit_by_id (private) =====

    #[test]
    fn edit_by_id_applies_closure() {
        let user = setup();
        user.add_category("Groceries").unwrap();
        let id = user.get_category("Groceries").unwrap();

        let query = user.categories().unwrap();
        let result = query.edit_by_id(id, |conn, id| {
            conn.execute(
                "UPDATE categories SET variant = ?1 WHERE id = ?2",
                rusqlite::params![CategoryVariant::Paired, id],
            )
            .map(|_| ())
        });
        assert!(result.is_ok());
        let refreshed = user.ls_category().unwrap();
        assert_eq!(refreshed[0].variant, CategoryVariant::Paired);
    }

    // ===== Transaction::edit_shared_field (private) =====

    #[test]
    fn edit_shared_field_updates_single_row_when_unlinked() {
        let user = setup();
        user.add_currency("MYR")
            .unwrap()
            .add_group("Food")
            .unwrap()
            .add_category("Groceries")
            .unwrap()
            .add_fund("Checking")
            .unwrap();
        user.add_transaction(
            "Lunch",
            None,
            (100, "MYR"),
            (1, 6, 2026),
            "Food",
            "Groceries",
            "Checking",
        )
        .unwrap();

        let query = user.transactions().unwrap();
        query
            .edit_shared_field(1, "name", "Dinner".to_string())
            .unwrap();

        let refreshed = user.ls_transaction().unwrap();
        assert_eq!(refreshed[0].label.name, "Dinner");
    }

    #[test]
    fn edit_shared_field_updates_both_rows_when_paired() {
        let user = setup();
        user.add_currency("MYR")
            .unwrap()
            .add_group("Transfer")
            .unwrap()
            .add_paired_category("Internal")
            .unwrap()
            .add_fund("Checking")
            .unwrap()
            .add_fund("Savings")
            .unwrap();
        user.add_paired_transaction(
            "Move",
            None,
            (50000, "MYR"),
            (50000, "MYR"),
            (1, 6, 2026),
            "Transfer",
            "Internal",
            "Checking",
            "Savings",
        )
        .unwrap();

        let query = user.transactions().unwrap();
        query
            .edit_shared_field(1, "name", "Relocated".to_string())
            .unwrap();

        let refreshed = user.ls_transaction().unwrap();
        assert!(
            refreshed.iter().all(|t| t.label.name == "Relocated"),
            "edit_shared_field should propagate to both halves of a paired transaction"
        );
    }

    #[test]
    fn edit_shared_field_invalid_index_errors() {
        let user = setup();
        user.add_currency("MYR")
            .unwrap()
            .add_group("Food")
            .unwrap()
            .add_category("Groceries")
            .unwrap()
            .add_fund("Checking")
            .unwrap();

        let query = user.transactions().unwrap(); // empty, no rows
        let result = query.edit_shared_field(1, "name", "X".to_string());
        assert!(result.is_err());
    }
}
