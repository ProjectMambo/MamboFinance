use crate::user::{
    Category, Currency, Date, Fund, Group, HasLabel, InputError, Label, Transaction, User,
    UserError, category::CategoryVariant,
};
use rusqlite::Connection;
use std::cmp::Ordering;
use std::collections::HashMap;
use uuid::Uuid;

/// A builder-like query wrapper for sorting, filtering, and removing ledger records.
pub struct Query<'a, T> {
    /// Reference to the active user context session.
    pub user: &'a User,
    /// The primary collection of visible dataset entries.
    pub rows: Vec<Entry<T>>,
    /// Internal buffer holding rows currently excluded by active filters.
    filtered_rows: Vec<Entry<T>>,

    /// The descriptive title of the query view.
    pub title: String,
    /// Configuration headers defining column names, widths, and structural variants.
    pub headers: Vec<(String, usize, FieldVariant)>,
}

// region: Type & Enum & Trait

/// Represents a single dataset row consisting of the underlying item, its 1-based index, and an optional linked positional reference.
type Entry<T> = (T, usize, Option<usize>);

/// A two-dimensional grid of strings representing flattened, printable tabular data.
type Matrix = Vec<Vec<String>>;

/// Variants representing structural formatting contexts for dataset columns.
#[derive(Copy, Clone)]
pub enum FieldVariant {
    /// A regular data column with fixed layout rules.
    Static,
    /// A column representing upper bound constraints or limits.
    Limit,
    /// A sequential, human-readable line numbering index.
    Index,
    /// A positional pointer referencing another corresponding entry.
    Link,
    /// A counter indicating the total number of associated sub-items.
    Count,
}

/// Provides a resetting mechanism to recalculate dynamic layout data.
pub trait Refreshable {
    /// Forces updates across internal structural properties and indexes.
    fn refresh(self) -> Self;
}

/// Allows an individual record item to be flattened into a sequential list of text cells.
pub trait Flattenable {
    /// Flattens the implementor into a vector of strings.
    fn flatten(&self) -> Vec<String>;
}

/// Allows an entire query collection to be flattened into a standard text matrix.
pub trait FlattenableQuery {
    /// Flattens the query view dataset into a printable two-dimensional string matrix.
    fn flatten(&self) -> Matrix;
}

// endregion

// region: General

impl<'a, T> Query<'a, T> {
    /// Returns the total number of currently visible rows in the dataset view.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns `true` if the query view contains no visible rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

// endregion

// region: New & Refresh

impl<'a, T: HasLabel> Query<'a, T>
where
    Query<'a, T>: Refreshable,
{
    /// Instantiates a new data view wrapper with applied layouts and baseline index sequences.
    pub fn new(
        user: &'a User,
        rows: Vec<T>,
        title: &str,
        headers: Vec<(&str, usize, FieldVariant)>,
    ) -> Self {
        let entries = rows
            .into_iter()
            .enumerate()
            .map(|(index, item)| (item, index + 1, None))
            .collect::<Vec<_>>();

        let headers: Vec<(String, usize, FieldVariant)> = headers
            .into_iter()
            .map(|(header, width, variant)| (header.to_string(), width, variant))
            .collect();

        Query {
            user,
            rows: entries,
            filtered_rows: Vec::new(),
            title: String::from(title),
            headers,
        }
        .refresh()
    }

    /// Sequentially recalculates human-facing layout enumeration indexes.
    fn refresh_index(mut self) -> Self {
        self.rows
            .iter_mut()
            .enumerate()
            .for_each(|(index, (_, idx, _))| *idx = index + 1);
        self
    }

    /// Calculates and scales column display dimensions safely based on item contents.
    fn refresh_width(mut self) -> Self {
        let index_digit: usize = self.rows.len().to_string().len();
        let opt_max: Option<usize> = self.rows.iter().filter_map(|(.., opt)| *opt).max();

        self.headers
            .iter_mut()
            .for_each(|(header, width, variant)| match variant {
                FieldVariant::Index => *width = index_digit.max(header.len()),
                FieldVariant::Link | FieldVariant::Count => {
                    *width = match opt_max {
                        Some(max) => max.to_string().len().max(header.len()),
                        None => header.len(),
                    }
                }
                _ => (),
            });
        self
    }

    /// Resolves mutually dependent entry pairs to map explicit item layout reference positions.
    fn refresh_link(mut self, link: impl Fn(&T) -> Option<Uuid>) -> Self {
        let id_to_index: HashMap<Uuid, usize> = self
            .rows
            .iter()
            .map(|(item, index, _)| (item.id(), *index))
            .collect();

        self.rows.iter_mut().for_each(|(item, _, link_index)| {
            *link_index = link(item).map(|l| id_to_index.get(&l).copied().unwrap_or(0));
        });
        self
    }

    /// Refreshes and attaches arbitrary counter metadata values to dataset entries.
    fn refresh_count(mut self, count: impl Fn(&T) -> usize) -> Self {
        self.rows.iter_mut().for_each(|(item, _, c)| {
            *c = Some(count(item));
        });
        self
    }
}

impl<'a> Refreshable for Query<'a, Group> {
    /// Triggers a structural layout and metadata refresh sequence for a Group dataset.
    fn refresh(self) -> Self {
        self.refresh_index()
            .refresh_count(|i| i.count)
            .refresh_width()
    }
}

impl<'a> Refreshable for Query<'a, Category> {
    /// Triggers a structural layout and metadata refresh sequence for a Category dataset.
    fn refresh(self) -> Self {
        self.refresh_index()
            .refresh_count(|i| i.count)
            .refresh_width()
    }
}

impl<'a> Refreshable for Query<'a, Fund> {
    /// Triggers a structural layout and metadata refresh sequence for a Fund dataset.
    fn refresh(self) -> Self {
        self.refresh_index()
            .refresh_count(|i| i.count)
            .refresh_width()
    }
}

impl<'a> Refreshable for Query<'a, Currency> {
    /// Triggers a structural layout and metadata refresh sequence for a Currency dataset.
    fn refresh(self) -> Self {
        self.refresh_index()
            .refresh_count(|i| i.count)
            .refresh_width()
    }
}

impl<'a> Refreshable for Query<'a, Transaction> {
    /// Triggers a structural layout and metadata refresh sequence for a Transaction dataset.
    fn refresh(self) -> Self {
        self.refresh_index()
            .refresh_link(|r| r.link)
            .refresh_width()
    }
}

// endregion

// region: Sort & Filter

impl<'a, T> Query<'a, T>
where
    Query<'a, T>: Refreshable,
{
    /// Reverses the current order of visible elements inside the query view.
    pub fn sort_reverse(mut self) -> Self {
        self.rows.reverse();
        self
    }

    /// Reverts the visible dataset back to its initialization state order.
    pub fn sort_clear(mut self) -> Self {
        self.rows.sort_by_key(|(_, index, _)| *index);
        self
    }

    /// Swaps matching and excluded rows to isolate non-matching elements.
    pub fn filter_reverse(mut self) -> Self {
        std::mem::swap(&mut self.rows, &mut self.filtered_rows);
        self
    }

    /// Appends partitioned records back to the main visibility vector and refreshes.
    pub fn filter_clear(mut self) -> Self {
        self.rows.extend(self.filtered_rows);
        self.filtered_rows = Vec::new();
        self.refresh()
    }

    /// Sorts internal record storage elements utilizing custom comparison logic functions.
    fn sort_by<F>(mut self, mut compare: F) -> Self
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.rows
            .sort_by(|(a_item, _, _), (b_item, _, _)| compare(a_item, b_item));
        self
    }

    /// Sorts internal record storage elements using generated sort extraction keys.
    fn sort_by_key<K, F>(mut self, mut f: F) -> Self
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.rows.sort_by_key(|(item, _, _)| f(item));
        self
    }

    /// Partitions matching rows from non-matching rows using a standard predicate filter.
    fn filter(mut self, mut part: impl FnMut(&T) -> bool) -> Self {
        let (matching, rejected): (Vec<_>, Vec<_>) =
            self.rows.into_iter().partition(|(item, ..)| part(item));

        self.rows = matching;
        self.filtered_rows.extend(rejected);
        self
    }
}

impl<'a, T: HasLabel> Query<'a, T>
where
    Query<'a, T>: Refreshable,
{
    /// Sorts the internal dataset alphabetically by name using lexical sorting order.
    pub fn sort_by_name(self) -> Self {
        self.sort_by(|a, b| a.name().cmp(b.name()))
    }
}

impl<'a> Query<'a, Category> {
    /// Sorts categories sequentially by their operational structure type (Single vs Paired).
    pub fn sort_by_type(self) -> Self {
        self.sort_by_key(|c| c.variant)
    }
}

impl<'a> Query<'a, Transaction> {
    /// Sorts transactions chronologically from oldest to newest.
    pub fn sort_by_date(self) -> Self {
        self.sort_by_key(|t| t.date)
    }

    /// Sorts transactions in ascending order by their numeric raw value.
    pub fn sort_by_amount(self) -> Self {
        self.sort_by_key(|t| t.amount.value)
    }

    /// Sorts transactions by absolute numerical values regardless of algebraic signs.
    pub fn sort_by_abs_amount(self) -> Self {
        self.sort_by_key(|t| t.amount.value.abs())
    }

    /// Partitions items by grouping inbound values separately from outbound flows.
    pub fn sort_by_flow(self) -> Self {
        self.sort_by_key(|t| t.amount.value < 0)
    }

    /// Sorts items lexically according to the descriptive label string of their tracking group.
    pub fn sort_by_group(self) -> Self {
        self.sort_by(|a, b| a.group.name().cmp(b.group.name()))
    }

    /// Sorts items lexically matching the identifier string values of their asset accounts.
    pub fn sort_by_fund(self) -> Self {
        self.sort_by(|a, b| a.fund.name().cmp(b.fund.name()))
    }

    /// Sorts records alphabetically using their descriptive tracking category strings.
    pub fn sort_by_category(self) -> Self {
        self.sort_by(|a, b| a.category.name().cmp(b.category.name()))
    }

    /// Sorts transactions alphabetically by the lexical name of their respective asset currencies.
    pub fn sort_by_currency(self) -> Self {
        self.sort_by(|a, b| a.amount.currency.name().cmp(b.amount.currency.name()))
    }

    /// Filters the internal collection to only contain transactions belonging to the given group.
    pub fn filter_group(self, group: &str) -> Self {
        self.filter(|t| t.group.name() == Label::fmt(group))
    }

    /// Filters the internal collection to only contain transactions tied to the given asset fund/account.
    pub fn filter_fund(self, fund: &str) -> Self {
        self.filter(|t| t.fund.name() == Label::fmt(fund))
    }

    /// Filters the internal collection to only contain transactions matching the given category.
    pub fn filter_category(self, category: &str) -> Self {
        self.filter(|t| t.category.name() == Label::fmt(category))
    }

    /// Filters the internal collection to only contain transactions using the specified currency asset type.
    pub fn filter_currency(self, currency: &str) -> Self {
        self.filter(|t| t.amount.currency.name() == Label::fmt(currency))
    }

    /// Constrains active view options to items recorded inside a bounding temporary date range.
    pub fn filter_date(self, (left, right): (Date, Date)) -> Self {
        self.filter(|t| left <= t.date && t.date < right)
    }
}

// endregion

// region: Get & Edit & Delete

impl<'a, T: HasLabel> Query<'a, T> {
    /// Resolves an exact context wrapper line matching its relational human row location.
    fn get_row(&self, index: usize) -> Result<&Entry<T>, UserError> {
        if index == 0 {
            return Err(UserError::Input(InputError::InvalidIndex(index)));
        }

        self.rows
            .get(index - 1)
            .ok_or(UserError::Input(InputError::InvalidIndex(index)))
    }

    /// Isolates and references an item value inside the specified indexed column entry array.
    fn get_item(&self, index: usize) -> Result<&T, UserError> {
        self.get_row(index).map(|(r, ..)| r)
    }

    /// Permanently deletes an entity from both database table storage and local view storage.
    ///
    /// # Errors
    ///
    /// Returns `UserError` if the index is out of bounds or database operations encounter issues.
    pub fn delete(mut self, index: usize) -> Result<Self, UserError> {
        let id = self.get_item(index)?.id();
        self.user
            .conn
            .execute(&format!("DELETE FROM {} WHERE id = ?1", T::table()), [id])
            .map_err(UserError::SQL)?;
        self.rows.retain(|(r, ..)| r.id() != id);
        Ok(self)
    }

    /// Standardized wrapper facilitating mutation operations while preserving structural integrity checks.
    fn edit(
        self,
        index: usize,
        existing: Option<&str>,
        edit: impl FnOnce(&rusqlite::Connection, Uuid) -> rusqlite::Result<usize>,
    ) -> Result<Self, UserError> {
        self.user.check_existing::<T>(existing)?;
        let id = self.get_item(index)?.id();
        edit(&self.user.conn, id)
            .map(|_| ())
            .map_err(UserError::SQL)?;
        Ok(self)
    }
}

impl<'a> Query<'a, Group> {
    /// Modifies a group's tracking label identifier, enforcing strict database uniqueness checks.
    pub fn edit_name(self, index: usize, new_name: &str) -> Result<Self, UserError> {
        let formatted_name = Label::fmt(new_name);
        self.edit(index, Some(&formatted_name), |conn, id| {
            conn.execute(
                "UPDATE groups SET name = ?1 WHERE id = ?2",
                rusqlite::params![formatted_name.clone(), id],
            )
        })
    }
}

impl<'a> Query<'a, Fund> {
    /// Modifies a fund account's string label identifier, enforcing strict database uniqueness checks.
    pub fn edit_name(self, index: usize, new_name: &str) -> Result<Self, UserError> {
        let formatted_name = Label::fmt(new_name);
        self.edit(index, Some(&formatted_name), |conn, id| {
            conn.execute(
                "UPDATE funds SET name = ?1 WHERE id = ?2",
                rusqlite::params![formatted_name.clone(), id],
            )
        })
    }
}

impl<'a> Query<'a, Category> {
    /// Modifies an accounting category's string label identifier, enforcing strict database uniqueness checks.
    pub fn edit_name(self, index: usize, new_name: &str) -> Result<Self, UserError> {
        let formatted_name = Label::fmt(new_name);
        self.edit(index, Some(&formatted_name), |conn, id| {
            conn.execute(
                "UPDATE categories SET name = ?1 WHERE id = ?2",
                rusqlite::params![formatted_name.clone(), id],
            )
        })
    }

    /// Toggles structural category variants (Single vs Paired), blocking variations with linked active records.
    pub fn edit_variant(
        self,
        index: usize,
        new_variant: CategoryVariant,
    ) -> Result<Self, UserError> {
        let cat = self.get_item(index)?;
        if cat.count > 0 {
            return Err(UserError::Input(InputError::CategoryInUse(cat.count)));
        }

        self.edit(index, None, |conn, id| {
            conn.execute(
                "UPDATE categories SET variant = ?1 WHERE id = ?2",
                rusqlite::params![new_variant, id],
            )
        })
    }

    /// Mutates structurally assigned category formats while forcing cascading unlinks over any active matching records.
    pub fn force_edit_variant(
        self,
        index: usize,
        new_variant: CategoryVariant,
    ) -> Result<Self, UserError> {
        self.edit(index, None, |conn, id| {
            conn.execute(
                "UPDATE transactions SET link_id = NULL WHERE category_id = ?1",
                [id],
            )
        })?
        .edit(index, None, |conn, id| {
            conn.execute(
                "UPDATE categories SET variant = ?1 WHERE id = ?2",
                rusqlite::params![new_variant, id],
            )
        })
    }
}

impl<'a> Query<'a, Currency> {
    /// Modifies a currency identity tracking symbol, enforcing strict database uniqueness checks.
    pub fn edit_name(self, index: usize, new_name: &str) -> Result<Self, UserError> {
        let formatted_name = Label::fmt(new_name);
        self.edit(index, Some(&formatted_name), |conn, id| {
            conn.execute(
                "UPDATE currencies SET name = ?1 WHERE id = ?2",
                rusqlite::params![formatted_name.clone(), id],
            )
        })
    }
}

impl<'a> Query<'a, Transaction> {
    /// Intercepts shared transactions to pass adjustments safely to double-entry system complements.
    fn edit_shared(
        self,
        index: usize,
        edit: impl FnOnce(&rusqlite::Connection, Uuid, Uuid) -> rusqlite::Result<usize>,
    ) -> Result<Self, UserError> {
        let link = self.get_item(index)?.link;
        let edit = |conn: &Connection, id| {
            let other_id = link.unwrap_or(id);
            edit(conn, id, other_id)
        };

        self.edit(index, None, edit)
    }

    /// Rewrites descriptive label identities for single transactions or double-entry pairs concurrently.
    pub fn edit_name(self, index: usize, new_name: &str) -> Result<Self, UserError> {
        let formatted_name = Label::fmt(new_name);
        self.edit_shared(index, |conn, id, other_id| {
            conn.execute(
                "UPDATE transactions SET name = ?1 WHERE id IN (?2, ?3)",
                rusqlite::params![formatted_name, id, other_id],
            )
        })
    }

    /// Remaps chosen transaction targets to a separate transaction group.
    pub fn edit_group(self, index: usize, new_group: &str) -> Result<Self, UserError> {
        let group_id = self.user.get::<Group>(&Label::fmt(new_group))?;
        self.edit_shared(index, |conn, id, other_id| {
            conn.execute(
                "UPDATE transactions SET group_id = ?1 WHERE id IN (?2, ?3)",
                rusqlite::params![group_id, id, other_id],
            )
        })
    }

    /// Modifies log timestamps across singular entries or linked dynamic double-entries.
    pub fn edit_date(self, index: usize, day: u8, month: u8, year: u16) -> Result<Self, UserError> {
        let date = Date::new(day, month, year)?;
        self.edit_shared(index, |conn, id, other_id| {
            conn.execute(
                "UPDATE transactions SET day=?1, month=?2, year=?3 WHERE id = ?4 OR id = ?5",
                rusqlite::params![date.day, date.month, date.year, id, other_id],
            )
        })
    }

    /// Alters operational categorization labels, requiring double-entry compatibility variants for paired targets.
    pub fn edit_category(self, index: usize, new_category: &str) -> Result<Self, UserError> {
        let category_id = self.user.get::<Category>(&Label::fmt(new_category))?;

        let required_variant = if self.get_item(index)?.link.is_some() {
            CategoryVariant::Paired
        } else {
            CategoryVariant::Single
        };
        self.user
            .check_category_variant(category_id, required_variant)?;

        self.edit_shared(index, |conn, id, other_id| {
            conn.execute(
                "UPDATE transactions SET category_id = ?1 WHERE id IN (?2, ?3)",
                rusqlite::params![category_id, id, other_id],
            )
        })
    }

    /// Adjusts individual account mappings for targeted transaction lines.
    pub fn edit_fund(self, index: usize, new_fund: &str) -> Result<Self, UserError> {
        let fund_id = self.user.get::<Fund>(&Label::fmt(new_fund))?;
        self.edit(index, None, |conn, id| {
            conn.execute(
                "UPDATE transactions SET fund_id = ?1 WHERE id = ?2",
                rusqlite::params![fund_id, id],
            )
        })
    }

    /// Updates raw financial values on a single target entry.
    pub fn edit_amount(self, index: usize, amount: i64) -> Result<Self, UserError> {
        self.edit(index, None, |conn, id| {
            conn.execute(
                "UPDATE transactions SET amount = ?1 WHERE id = ?2",
                rusqlite::params![amount, id],
            )
        })
    }

    /// Adjusts systemic currency mapping associations on a single target entry.
    pub fn edit_currency(self, index: usize, new_currency: &str) -> Result<Self, UserError> {
        let currency_id = self.user.get::<Currency>(&Label::fmt(new_currency))?;
        self.edit(index, None, |conn, id| {
            conn.execute(
                "UPDATE transactions SET currency_id = ?1 WHERE id = ?2",
                rusqlite::params![currency_id, id],
            )
        })
    }
}

// endregion

// region: Flat

impl<'a, T: Flattenable> Query<'a, T> {
    /// Shared utility function facilitating tabular row transformations.
    fn flatten_helper<F>(&self, handle_opt: F) -> Matrix
    where
        F: Fn(&Option<usize>) -> String,
    {
        self.rows
            .iter()
            .map(|(item, idx, opt)| {
                let mut row = vec![idx.to_string()];
                row.extend(item.flatten());
                row.push(handle_opt(opt));
                row
            })
            .collect()
    }
}

impl<'a> FlattenableQuery for Query<'a, Group> {
    /// Flattens a Group query dataset into an exportable matrix format.
    fn flatten(&self) -> Matrix {
        self.flatten_helper(|opt| match opt {
            Some(o) => o.to_string(),
            None => String::from("0"),
        })
    }
}

impl<'a> FlattenableQuery for Query<'a, Category> {
    /// Flattens a Category query dataset into an exportable matrix format.
    fn flatten(&self) -> Matrix {
        self.flatten_helper(|opt| match opt {
            Some(o) => o.to_string(),
            None => String::from("0"),
        })
    }
}

impl<'a> FlattenableQuery for Query<'a, Fund> {
    /// Flattens a Fund query dataset into an exportable matrix format.
    fn flatten(&self) -> Matrix {
        self.flatten_helper(|opt| match opt {
            Some(o) => o.to_string(),
            None => String::from("0"),
        })
    }
}

impl<'a> FlattenableQuery for Query<'a, Currency> {
    /// Flattens a Currency query dataset into an exportable matrix format.
    fn flatten(&self) -> Matrix {
        self.flatten_helper(|opt| match opt {
            Some(o) => o.to_string(),
            None => String::from("0"),
        })
    }
}

impl<'a> FlattenableQuery for Query<'a, Transaction> {
    /// Flattens a Transaction query dataset into an exportable matrix format.
    fn flatten(&self) -> Matrix {
        self.flatten_helper(|opt| match opt {
            Some(0) => String::from("OOR"),
            Some(o) => o.to_string(),
            None => String::from("-"),
        })
    }
}

// endregion

// region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::User;

    // region: helpers

    /// Builds an in-memory User pre-populated with one group, one fund, one currency,
    /// a single-entry category, and a paired-entry category, ready for transaction setup.
    fn seeded_user() -> User {
        let user = User::new_in_memory("test").expect("failed to create in-memory user");
        user.add_group("Personal").expect("add_group failed");
        user.add_fund("Cash").expect("add_fund failed");
        user.add_fund("Bank").expect("add_fund failed");
        user.add_currency("USD").expect("add_currency failed");
        user.add_category("Food").expect("add_category failed");
        user.add_category("Transport").expect("add_category failed");
        user.add_paired_category("Transfer")
            .expect("add_paired_category failed");
        user
    }

    /// Adds a single-entry transaction with sensible defaults, varying only what's passed in.
    fn add_tx(
        user: &User,
        name: &str,
        amount: i64,
        (day, month, year): (u8, u8, u16),
        group: &str,
        category: &str,
        fund: &str,
    ) {
        user.add_transaction(
            name,
            None,
            (amount, "USD"),
            (day, month, year),
            group,
            category,
            fund,
        )
        .expect("add_transaction failed");
    }

    // endregion

    // region: Query::new & refresh internals

    /// Verifies that instantiating a new query assigns sequential, 1-based indexing values
    /// to all items loaded into the collection view.
    #[test]
    fn new_assigns_sequential_one_based_indexes() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "B", 200, (2, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "C", 300, (3, 1, 2026), "Personal", "Food", "Cash");

        // Act
        let query = user.transactions().expect("transactions query failed");

        // Assert
        let indexes: Vec<usize> = query.rows.iter().map(|(_, idx, _)| *idx).collect();
        assert_eq!(indexes, vec![1, 2, 3]);
    }

    /// Ensures that a newly initialized query wrapper contains an entirely empty secondary buffer
    /// for tracking excluded or partitioned dataset records.
    #[test]
    fn new_starts_with_an_empty_filtered_rows_buffer() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");

        // Act
        let query = user.transactions().expect("transactions query failed");

        // Assert
        assert!(query.filtered_rows.is_empty());
    }

    /// Verifies that renumbering sequential indexing structures correctly recalculates human-facing
    /// display values after dataset ordering adjustments or mutations occur.
    #[test]
    fn refresh_index_renumbers_rows_sequentially_after_reordering() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "B", 200, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let refreshed = query.sort_by_name().sort_clear();

        // Assert
        let indexes: Vec<usize> = refreshed.rows.iter().map(|(_, idx, _)| *idx).collect();
        assert_eq!(indexes, vec![1, 2]);
    }

    /// Confirms that link resolution logic correctly discovers and tracks corresponding line numbers
    /// between mutual double-entry balancing lines in a transaction query context.
    #[test]
    fn refresh_link_resolves_link_index_for_paired_transactions() {
        // Arrange
        let user = seeded_user();
        user.add_paired_transaction(
            "Move",
            None,
            (500, "USD"),
            (500, "USD"),
            (1, 1, 2026),
            "Personal",
            "Transfer",
            "Cash",
            "Bank",
        )
        .expect("add_paired_transaction failed");

        // Act
        let query = user.transactions().expect("transactions query failed");

        // Assert
        let link_indexes: Vec<Option<usize>> =
            query.rows.iter().map(|(_, _, link)| *link).collect();
        assert_eq!(link_indexes, vec![Some(2), Some(1)]);
    }

    /// Assures that individual single-entry rows do not incorrectly assign or discover arbitrary link metadata
    /// if they have no corresponding second half in a double-entry balance scheme.
    #[test]
    fn refresh_link_yields_none_for_unlinked_single_transactions() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");

        // Act
        let query = user.transactions().expect("transactions query failed");

        // Assert
        assert_eq!(query.rows[0].2, None);
    }

    // endregion

    // region: sort_reverse & sort_clear

    /// Checks that the internal structural sequencing of visible rows can be cleanly inverted
    /// through a standard reversal operation.
    #[test]
    fn sort_reverse_flips_row_order() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "B", 200, (2, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let reversed = query.sort_reverse();

        // Assert
        let indexes: Vec<usize> = reversed.rows.iter().map(|(_, idx, _)| *idx).collect();
        assert_eq!(indexes, vec![2, 1]);
    }

    /// Verifies that clearing sort states restores the records precisely to the baseline sorting sequence
    /// assigned during original context initialization.
    #[test]
    fn sort_clear_restores_original_initialization_order() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "B", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "A", 200, (2, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let restored = query.sort_by_name().sort_clear();

        // Assert
        let indexes: Vec<usize> = restored.rows.iter().map(|(_, idx, _)| *idx).collect();
        assert_eq!(indexes, vec![1, 2]);
    }

    // endregion

    // region: filter_reverse & filter_clear

    /// Assures that swapping the primary visibility vector with the excluded partition buffer efficiently
    /// isolates the elements that previously failed predicate filtration.
    #[test]
    fn filter_reverse_swaps_matching_and_filtered_buffers() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(
            &user,
            "B",
            200,
            (1, 1, 2026),
            "Personal",
            "Transport",
            "Cash",
        );
        let query = user.transactions().expect("transactions query failed");

        // Act
        let filtered = query.filter_category("Food");
        let total_before_reverse = filtered.rows.len();
        let reversed = filtered.filter_reverse();

        // Assert
        assert_eq!(total_before_reverse, 1);
        assert_eq!(reversed.rows.len(), 1);
    }

    /// Verifies that resetting active filter predicates updates overall ledger layout properties and smoothly
    /// re-incorporates previously partitioned entries back into view.
    #[test]
    fn filter_clear_merges_filtered_rows_back_and_refreshes_indexes() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(
            &user,
            "B",
            200,
            (1, 1, 2026),
            "Personal",
            "Transport",
            "Cash",
        );
        let query = user.transactions().expect("transactions query failed");

        // Act
        let cleared = query.filter_category("Food").filter_clear();

        // Assert
        assert_eq!(cleared.rows.len(), 2);
        assert!(cleared.filtered_rows.is_empty());
        let indexes: Vec<usize> = cleared.rows.iter().map(|(_, idx, _)| *idx).collect();
        assert_eq!(indexes, vec![1, 2]);
    }

    // endregion

    // region: sort_by_name (generic, HasLabel-bound)

    /// Verifies that entities implementing generic text labeling sort alphabetically using proper
    /// lexical collation logic rules.
    #[test]
    fn sort_by_name_orders_groups_lexically() {
        // Arrange
        let user = seeded_user();
        user.add_group("Zulu").expect("add_group failed");
        user.add_group("Alpha").expect("add_group failed");

        // Act
        let sorted = user.groups().expect("groups query failed").sort_by_name();

        // Assert
        let names: Vec<&str> = sorted.rows.iter().map(|(item, ..)| item.name()).collect();
        assert_eq!(names, vec!["Alpha", "Personal", "Zulu"]);
    }

    // endregion

    // region: Category::sort_by_type

    /// Confirms that accounting category entities organize predictably with non-linked individual
    /// configurations sorting cleanly ahead of paired double-entry variants.
    #[test]
    fn sort_by_type_orders_single_before_paired() {
        // Arrange
        let user = seeded_user();
        let query = user
            .categories()
            .expect("categories query failed")
            .sort_reverse();

        // Act
        let sorted = query.sort_by_type();

        // Assert
        let variants: Vec<CategoryVariant> =
            sorted.rows.iter().map(|(item, ..)| item.variant).collect();
        let mut sorted_variants = variants.clone();
        sorted_variants.sort();
        assert_eq!(variants, sorted_variants);
        assert_eq!(*variants.last().unwrap(), CategoryVariant::Paired);
    }

    // endregion

    // region: Transaction sort_by_*

    /// Confirms that transaction entries arrange sequentially based on exact date markers, ordering oldest elements
    /// into earliest positions.
    #[test]
    fn sort_by_date_orders_oldest_first() {
        // Arrange
        let user = seeded_user();
        add_tx(
            &user,
            "Later",
            100,
            (20, 6, 2026),
            "Personal",
            "Food",
            "Cash",
        );
        add_tx(
            &user,
            "Earlier",
            100,
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        );
        let query = user.transactions().expect("transactions query failed");

        // Act
        let sorted = query.sort_by_date();

        // Assert
        assert_eq!(sorted.rows.len(), 2);
        assert_eq!(sorted.rows[0].1, 2);
    }

    /// Checks that records order themselves sequentially by cash valuation metrics, placing deeper negative values
    /// ahead of progressive positive inflows.
    #[test]
    fn sort_by_amount_orders_ascending_including_sign() {
        // Arrange
        let user = seeded_user();
        add_tx(
            &user,
            "Positive",
            500,
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        );
        add_tx(
            &user,
            "Negative",
            -200,
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        );
        let query = user.transactions().expect("transactions query failed");

        // Act
        let sorted = query.sort_by_amount();

        // Assert
        assert_eq!(sorted.rows[0].1, 2);
    }

    /// Verifies that sorting transactions by absolute amounts targets magnitude differences only, ignoring signs.
    #[test]
    fn sort_by_abs_amount_ignores_sign() {
        // Arrange
        let user = seeded_user();
        add_tx(
            &user,
            "Large",
            200,
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        );
        add_tx(
            &user,
            "Small",
            -50,
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        );
        let query = user.transactions().expect("transactions query failed");

        // Act
        let sorted = query.sort_by_abs_amount();

        // Assert
        assert_eq!(sorted.rows[0].1, 2);
        assert_eq!(sorted.rows[1].1, 1);
    }

    /// Validates that flow sorting logic correctly separates negative disbursements from inbound ledger changes.
    #[test]
    fn sort_by_flow_groups_outgoing_before_incoming_when_inserted_in_reverse() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "Out", -500, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "In", 500, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let sorted = query.sort_by_flow();

        // Assert
        assert_eq!(sorted.rows[0].1, 2);
        assert_eq!(sorted.rows[1].1, 1);
    }

    /// Ensures transaction entries resolve alphabetic relationships based on text keys extracted
    /// from assigned parent tracking groups.
    #[test]
    fn sort_by_group_orders_lexically_by_group_name() {
        // Arrange
        let user = seeded_user();
        user.add_group("Business").expect("add_group failed");
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "B", 100, (1, 1, 2026), "Business", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let sorted = query.sort_by_group();

        // Assert
        assert_eq!(sorted.rows[0].1, 2);
    }

    /// Assures transaction sorting logic aggregates records alphabetically matching lexical identifiers
    /// mapped to asset storage funds.
    #[test]
    fn sort_by_fund_orders_lexically_by_fund_name() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "B", 100, (1, 1, 2026), "Personal", "Food", "Bank");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let sorted = query.sort_by_fund();

        // Assert
        assert_eq!(sorted.rows[0].1, 2);
        assert_eq!(sorted.rows[1].1, 1);
    }

    /// Assures transaction records collate alphabetically based on formatting values stored inside
    /// operational tracking categories.
    #[test]
    fn sort_by_category_orders_lexically_by_category_name() {
        // Arrange
        let user = seeded_user();
        add_tx(
            &user,
            "A",
            100,
            (1, 1, 2026),
            "Personal",
            "Transport",
            "Cash",
        );
        add_tx(&user, "B", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let sorted = query.sort_by_category();

        // Assert
        assert_eq!(sorted.rows[0].1, 2);
    }

    /// Confirms transactions sort alphabetically according to naming keys resolved on the currency definition blocks.
    #[test]
    fn sort_by_currency_orders_lexically_by_currency_name() {
        // Arrange
        let user = seeded_user();
        user.add_currency("EUR").expect("add_currency failed");
        user.add_transaction(
            "A",
            None,
            (100, "USD"),
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        )
        .expect("add_transaction failed");
        user.add_transaction(
            "B",
            None,
            (100, "EUR"),
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        )
        .expect("add_transaction failed");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let sorted = query.sort_by_currency();

        // Assert
        assert_eq!(sorted.rows[0].1, 2);
    }

    // endregion

    // region: Transaction filter_*

    /// Verifies group filter selectors isolate matching entries while shifting non-matching lines to the filter buffer.
    #[test]
    fn filter_group_keeps_only_matching_group_rows() {
        // Arrange
        let user = seeded_user();
        user.add_group("Business").expect("add_group failed");
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "B", 100, (1, 1, 2026), "Business", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let filtered = query.filter_group("Personal");

        // Assert
        assert_eq!(filtered.rows.len(), 1);
        assert_eq!(filtered.filtered_rows.len(), 1);
    }

    /// Confirms fund filter actions reject lines not associated explicitly with the designated financial account.
    #[test]
    fn filter_fund_keeps_only_matching_fund_rows() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "B", 100, (1, 1, 2026), "Personal", "Food", "Bank");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let filtered = query.filter_fund("Cash");

        // Assert
        assert_eq!(filtered.rows.len(), 1);
    }

    /// Validates category constraints remove items that do not specify the designated category tracking name.
    #[test]
    fn filter_category_keeps_only_matching_category_rows() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(
            &user,
            "B",
            100,
            (1, 1, 2026),
            "Personal",
            "Transport",
            "Cash",
        );
        let query = user.transactions().expect("transactions query failed");

        // Act
        let filtered = query.filter_category("Transport");

        // Assert
        assert_eq!(filtered.rows.len(), 1);
    }

    /// Verifies currency filter constraints accurately discard entries recorded under differing asset formats.
    #[test]
    fn filter_currency_keeps_only_matching_currency_rows() {
        // Arrange
        let user = seeded_user();
        user.add_currency("EUR").expect("add_currency failed");
        user.add_transaction(
            "A",
            None,
            (100, "USD"),
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        )
        .expect("add_transaction failed");
        user.add_transaction(
            "B",
            None,
            (100, "EUR"),
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        )
        .expect("add_transaction failed");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let filtered = query.filter_currency("EUR");

        // Assert
        assert_eq!(filtered.rows.len(), 1);
    }

    /// Confirms calendar date window filtration blocks items outside the half-open temporal boundaries.
    #[test]
    fn filter_date_keeps_rows_within_the_half_open_range() {
        // Arrange
        let user = seeded_user();
        add_tx(
            &user,
            "Before",
            100,
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        );
        add_tx(
            &user,
            "Inside",
            100,
            (15, 6, 2026),
            "Personal",
            "Food",
            "Cash",
        );
        add_tx(
            &user,
            "OnEdge",
            100,
            (1, 1, 2027),
            "Personal",
            "Food",
            "Cash",
        );
        let query = user.transactions().expect("transactions query failed");

        // Act
        let left = Date::new(1, 6, 2026).expect("valid date");
        let right = Date::new(1, 1, 2027).expect("valid date");
        let filtered = query.filter_date((left, right));

        // Assert
        assert_eq!(filtered.rows.len(), 1);
    }

    // endregion

    // region: get_row & get_item (via delete/edit error paths)

    /// Ensures indexing calculations reject line requests explicitly addressing row zero with an input validation error.
    #[test]
    fn get_row_rejects_index_zero() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.delete(0);

        // Assert
        assert!(matches!(
            result,
            Err(UserError::Input(InputError::InvalidIndex(0)))
        ));
    }

    /// Assures indexing checks detect and block lookups addressing values past current limits.
    #[test]
    fn get_row_rejects_index_beyond_row_count() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.delete(99);

        // Assert
        assert!(matches!(
            result,
            Err(UserError::Input(InputError::InvalidIndex(99)))
        ));
    }

    /// Confirms valid layout identifiers pull back the expected ledger entry reference from row storage.
    #[test]
    fn get_item_resolves_the_correct_row_at_a_valid_index() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.delete(1);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows.len(), 0);
    }

    // endregion

    // region: delete

    /// Validates record deletion actions wipe underlying records from internal vector layouts
    /// and commit removals directly to table spaces.
    #[test]
    fn delete_removes_the_row_and_persists_the_deletion_in_the_database() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "B", 200, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let after_delete = query.delete(1).expect("delete should succeed");

        // Assert
        assert_eq!(after_delete.rows.len(), 1);
        let refetched = user.transactions().expect("transactions query failed");
        assert_eq!(refetched.rows.len(), 1);
    }

    // endregion

    // region: edit (via Group/Fund/Category/Currency edit_name)

    /// Verifies group text labels update reliably in the database when mutation requests pass unique properties.
    #[test]
    fn edit_updates_the_group_name_in_the_database() {
        // Arrange
        let user = seeded_user();
        let query = user.groups().expect("groups query failed");
        let target_index = query
            .rows
            .iter()
            .position(|(item, ..)| item.name() == "Personal")
            .map(|p| p + 1)
            .expect("Personal group should exist");

        // Act
        let result = query.edit_name(target_index, "Family");

        // Assert
        assert!(result.is_ok());
        let refetched = user.groups().expect("groups query failed");
        assert!(
            refetched
                .rows
                .iter()
                .any(|(item, ..)| item.name() == "Family")
        );
    }

    /// Ensures unique tracking structures block modifications that create naming duplicates.
    #[test]
    fn edit_rejects_renaming_to_an_existing_name() {
        // Arrange
        let user = seeded_user();
        user.add_group("Business").expect("add_group failed");
        let query = user.groups().expect("groups query failed");
        let personal_index = query
            .rows
            .iter()
            .position(|(item, ..)| item.name() == "Personal")
            .map(|p| p + 1)
            .expect("Personal group should exist");

        // Act
        let result = query.edit_name(personal_index, "Business");

        // Assert
        assert!(matches!(
            result,
            Err(UserError::Input(InputError::ExistingItem(_, _)))
        ));
    }

    /// Assures asset fund accounts change descriptive metadata labels successfully following valid update actions.
    #[test]
    fn edit_name_updates_fund_name() {
        // Arrange
        let user = seeded_user();
        let query = user.funds().expect("funds query failed");
        let cash_index = query
            .rows
            .iter()
            .position(|(item, ..)| item.name() == "Cash")
            .map(|p| p + 1)
            .expect("Cash fund should exist");

        // Act
        let result = query.edit_name(cash_index, "Wallet");

        // Assert
        assert!(result.is_ok());
        let refetched = user.funds().expect("funds query failed");
        assert!(
            refetched
                .rows
                .iter()
                .any(|(item, ..)| item.name() == "Wallet")
        );
    }

    /// Verifies tracking category entities apply descriptive string mutations accurately throughout back-end engines.
    #[test]
    fn edit_name_updates_category_name() {
        // Arrange
        let user = seeded_user();
        let query = user.categories().expect("categories query failed");
        let food_index = query
            .rows
            .iter()
            .position(|(item, ..)| item.name() == "Food")
            .map(|p| p + 1)
            .expect("Food category should exist");

        // Act
        let result = query.edit_name(food_index, "Groceries");

        // Assert
        assert!(result.is_ok());
        let refetched = user.categories().expect("categories query failed");
        assert!(
            refetched
                .rows
                .iter()
                .any(|(item, ..)| item.name() == "Groceries")
        );
    }

    /// Validates system asset currencies apply text adjustments predictably upon processing rename procedures.
    #[test]
    fn edit_name_updates_currency_name() {
        // Arrange
        let user = seeded_user();
        let query = user.currencies().expect("currencies query failed");
        let usd_index = query
            .rows
            .iter()
            .position(|(item, ..)| item.name() == "USD")
            .map(|p| p + 1)
            .expect("USD currency should exist");

        // Act
        let result = query.edit_name(usd_index, "Dollar");

        // Assert
        assert!(result.is_ok());
        let refetched = user.currencies().expect("currencies query failed");
        assert!(
            refetched
                .rows
                .iter()
                .any(|(item, ..)| item.name() == "Dollar")
        );
    }

    // endregion

    // region: Category::edit_variant & force_edit_variant

    /// Confirms category entities rewrite structural behaviors (Single vs Paired) smoothly if they have
    /// no active ledger transaction lines mapped to them.
    #[test]
    fn edit_variant_changes_variant_when_unused() {
        // Arrange
        let user = seeded_user();
        let query = user.categories().expect("categories query failed");
        let food_index = query
            .rows
            .iter()
            .position(|(item, ..)| item.name() == "Food")
            .map(|p| p + 1)
            .expect("Food category should exist");

        // Act
        let result = query.edit_variant(food_index, CategoryVariant::Paired);

        // Assert
        assert!(result.is_ok());
        let refetched = user.categories().expect("categories query failed");
        let food = refetched
            .rows
            .iter()
            .find(|(item, ..)| item.name() == "Food")
            .expect("Food category should still exist");
        assert_eq!(food.0.variant, CategoryVariant::Paired);
    }

    /// Confirms structural category changes fail immediately with a validation error if active transactions
    /// are mapped to the targeted entity.
    #[test]
    fn edit_variant_rejects_change_when_category_is_in_use() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.categories().expect("categories query failed");
        let food_index = query
            .rows
            .iter()
            .position(|(item, ..)| item.name() == "Food")
            .map(|p| p + 1)
            .expect("Food category should exist");

        // Act
        let result = query.edit_variant(food_index, CategoryVariant::Paired);

        // Assert
        assert!(matches!(
            result,
            Err(UserError::Input(InputError::CategoryInUse(1)))
        ));
    }

    /// Verifies cascading updates unlink structural connections on matched records while successfully remapping
    /// the targeted category variant.
    #[test]
    fn force_edit_variant_unlinks_transactions_and_changes_variant() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.categories().expect("categories query failed");
        let food_index = query
            .rows
            .iter()
            .position(|(item, ..)| item.name() == "Food")
            .map(|p| p + 1)
            .expect("Food category should exist");

        // Act
        let result = query.force_edit_variant(food_index, CategoryVariant::Paired);

        // Assert
        assert!(result.is_ok());
        let refetched = user.categories().expect("categories query failed");
        let food = refetched
            .rows
            .iter()
            .find(|(item, ..)| item.name() == "Food")
            .expect("Food category should still exist");
        assert_eq!(food.0.variant, CategoryVariant::Paired);
    }

    // endregion

    // region: Transaction edit_shared & edit_name/edit_group/edit_date/edit_category/edit_fund/edit_amount/edit_currency

    /// Confirms that altering singular transaction lines safely applies descriptive changes without disrupting other items.
    #[test]
    fn edit_name_updates_a_single_transaction() {
        // Arrange
        let user = seeded_user();
        add_tx(
            &user,
            "Old Name",
            100,
            (1, 1, 2026),
            "Personal",
            "Food",
            "Cash",
        );
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_name(1, "New Name");

        // Assert
        assert!(result.is_ok());
        let refetched = user.transactions().expect("transactions query failed");
        assert_eq!(refetched.rows[0].0.label.name, "New Name");
    }

    /// Verifies that modifying the name of a multi-entry/paired transaction concurrently updates
    /// the description label on both the debit and credit sides of the ledger entry.
    #[test]
    fn edit_name_updates_both_sides_of_a_paired_transaction() {
        // Arrange
        let user = seeded_user();
        user.add_paired_transaction(
            "Old Name",
            None,
            (500, "USD"),
            (500, "USD"),
            (1, 1, 2026),
            "Personal",
            "Transfer",
            "Cash",
            "Bank",
        )
        .expect("add_paired_transaction failed");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_name(1, "New Name");

        // Assert
        assert!(result.is_ok());
        let refetched = user.transactions().expect("transactions query failed");
        assert!(
            refetched
                .rows
                .iter()
                .all(|(t, ..)| t.label.name == "New Name")
        );
    }

    /// Assures transaction records remap successfully to separate operational tracking groups.
    #[test]
    fn edit_group_remaps_a_transaction_to_a_new_group() {
        // Arrange
        let user = seeded_user();
        user.add_group("Business").expect("add_group failed");
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_group(1, "Business");

        // Assert
        assert!(result.is_ok());
        let refetched = user.transactions().expect("transactions query failed");
        assert_eq!(refetched.rows[0].0.group.label.name, "Business");
    }

    /// Confirms ledger entry lines apply chronological date modifications cleanly when provided structurally valid inputs.
    #[test]
    fn edit_date_updates_the_transaction_date() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_date(1, 25, 12, 2026);

        // Assert
        assert!(result.is_ok());
        let refetched = user.transactions().expect("transactions query failed");
        let date = refetched.rows[0].0.date;
        assert_eq!((date.day, date.month, date.year), (25, 12, 2026));
    }

    /// Verifies calendar adjustments catch and block illogical inputs via date validation errors.
    #[test]
    fn edit_date_rejects_an_invalid_date() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_date(1, 31, 4, 2026);

        // Assert
        assert!(matches!(result, Err(UserError::Date(_))));
    }

    /// Confirms target category fields rewrite properly on individual lines upon processing valid updates.
    #[test]
    fn edit_category_updates_a_single_transactions_category() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_category(1, "Transport");

        // Assert
        assert!(result.is_ok());
        let refetched = user.transactions().expect("transactions query failed");
        assert_eq!(refetched.rows[0].0.category.label.name, "Transport");
    }

    /// Assures double-entry pairings reject category remapping requests if the targeted category structure
    /// lacks support for balanced paired definitions.
    #[test]
    fn edit_category_rejects_paired_transaction_moved_to_single_category() {
        // Arrange
        let user = seeded_user();
        user.add_paired_transaction(
            "Move",
            None,
            (500, "USD"),
            (500, "USD"),
            (1, 1, 2026),
            "Personal",
            "Transfer",
            "Cash",
            "Bank",
        )
        .expect("add_paired_transaction failed");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_category(1, "Food");

        // Assert
        assert!(matches!(
            result,
            Err(UserError::Input(InputError::WrongVariant(_)))
        ));
    }

    /// Validates fund storage locations update reliably on specific individual transaction elements.
    #[test]
    fn edit_fund_remaps_a_transaction_to_a_new_fund() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_fund(1, "Bank");

        // Assert
        assert!(result.is_ok());
        let refetched = user.transactions().expect("transactions query failed");
        assert_eq!(refetched.rows[0].0.fund.label.name, "Bank");
    }

    /// Checks that numeric changes safely rewrite underlying financial raw values.
    #[test]
    fn edit_amount_updates_the_raw_value() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_amount(1, 999);

        // Assert
        assert!(result.is_ok());
        let refetched = user.transactions().expect("transactions query failed");
        assert_eq!(refetched.rows[0].0.amount.value, 999);
    }

    /// Confirms asset currency definitions rewrite precisely on single targeted lines following modification calls.
    #[test]
    fn edit_currency_remaps_a_transaction_to_a_new_currency() {
        // Arrange
        let user = seeded_user();
        user.add_currency("EUR").expect("add_currency failed");
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_currency(1, "EUR");

        // Assert
        assert!(result.is_ok());
        let refetched = user.transactions().expect("transactions query failed");
        assert_eq!(refetched.rows[0].0.amount.currency.label.name, "EUR");
    }

    // endregion
}

// endregion
