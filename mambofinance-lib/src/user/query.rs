use crate::user::{
    Category, Currency, Date, Fund, Group, HasLabel, InputError, Label, Transaction, User,
    UserError, category::CategoryVariant,
};
use rusqlite::Connection;
use std::cmp::Ordering;
use std::collections::HashMap;
use uuid::Uuid;

/// A builder-like query wrapper for sorting, filtering, displaying, and removing ledger records.
pub struct Query<'a, T> {
    pub user: &'a User,
    pub rows: Vec<Entry<T>>,
    filtered_rows: Vec<Entry<T>>,

    pub title: String,
    pub headers: Vec<(String, usize, FieldVariant)>,
}

// region: Type & Enum & Trait

type Entry<T> = (T, usize, Option<usize>);

/// Variants representing structural formatting contexts for dataset columns.
pub enum FieldVariant {
    Static,
    Index,
    Link,
    Count,
}

/// Provides a resetting mechanism to recalculate dynamic layout data.
pub trait Refreshable {
    /// Forces updates across internal structural properties and indexes.
    fn refresh(self) -> Self;
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

    // Sequentially recalculates human-facing layout enumeration indexes.
    fn refresh_index(mut self) -> Self {
        self.rows
            .iter_mut()
            .enumerate()
            .for_each(|(index, (_, idx, _))| *idx = index + 1);
        self
    }

    // Calculates and scales column display dimensions safely based on item contents.
    fn refresh_width(mut self) -> Self {
        let index_digit: usize = self.rows.len().to_string().len();
        let opt_max: Option<usize> = self.rows.iter().filter_map(|(.., opt)| *opt).max();

        self.headers
            .iter_mut()
            .for_each(|(header, width, variant)| match variant {
                FieldVariant::Index => *width = index_digit.max(header.len()),
                FieldVariant::Link | FieldVariant::Count => {
                    *width = match opt_max {
                        Some(max) => max.max(header.len()),
                        None => header.len(),
                    }
                }
                _ => (),
            });
        self
    }

    // Resolves mutually dependent entry pairs to map explicit item layout reference positions.
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
}

impl<'a> Refreshable for Query<'a, Group> {
    fn refresh(self) -> Self {
        self.refresh_index().refresh_width()
    }
}

impl<'a> Refreshable for Query<'a, Category> {
    fn refresh(self) -> Self {
        self.refresh_index().refresh_width()
    }
}

impl<'a> Refreshable for Query<'a, Fund> {
    fn refresh(self) -> Self {
        self.refresh_index().refresh_width()
    }
}

impl<'a> Refreshable for Query<'a, Currency> {
    fn refresh(self) -> Self {
        self.refresh_index().refresh_width()
    }
}

impl<'a> Refreshable for Query<'a, Transaction> {
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

    // Sorts internal record storage elements utilizing custom comparison logic functions.
    fn sort_by<F>(mut self, mut compare: F) -> Self
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.rows
            .sort_by(|(a_item, _, _), (b_item, _, _)| compare(a_item, b_item));
        self
    }

    // Sorts internal record storage elements using generated sort extraction keys.
    fn sort_by_key<K, F>(mut self, mut f: F) -> Self
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.rows.sort_by_key(|(item, _, _)| f(item));
        self
    }

    // Partitions matching rows from non-matching rows using a standard predicate filter.
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
    // Resolves an exact context wrapper line matching its relational human row location.
    fn get_row(&self, index: usize) -> Result<&Entry<T>, UserError> {
        if index == 0 {
            return Err(UserError::Input(InputError::InvalidIndex(index)));
        }

        self.rows
            .get(index - 1)
            .ok_or(UserError::Input(InputError::InvalidIndex(index)))
    }

    // Isolates and references an item value inside the specified indexed column entry array.
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

    // Standardized wrapper facilitating mutation operations while preserving structural integrity checks.
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
    // Intercepts shared transactions to pass adjustments safely to double-entry system complements.
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

//  region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::User;

    // region: helpers

    // Builds an in-memory User pre-populated with one group, one fund, one currency,
    // a single-entry category, and a paired-entry category, ready for transaction setup.
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

    // Adds a single-entry transaction with sensible defaults, varying only what's passed in.
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

    #[test]
    fn refresh_index_renumbers_rows_sequentially_after_reordering() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "B", 200, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let refreshed = query.sort_by_name().sort_clear(); // sort_clear re-sorts by stored index, then we re-derive index

        // Assert
        let indexes: Vec<usize> = refreshed.rows.iter().map(|(_, idx, _)| *idx).collect();
        assert_eq!(indexes, vec![1, 2]);
    }

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
        // Both sides of the pair should resolve a link_index pointing at each other.
        let link_indexes: Vec<Option<usize>> =
            query.rows.iter().map(|(_, _, link)| *link).collect();
        assert_eq!(link_indexes, vec![Some(2), Some(1)]);
    }

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

    #[test]
    fn refresh_width_sizes_index_column_to_row_count_digit_length() {
        // Arrange
        let user = seeded_user();
        for i in 0..12 {
            add_tx(
                &user,
                &format!("Item{i}"),
                100,
                (1, 1, 2026),
                "Personal",
                "Food",
                "Cash",
            );
        }

        // Act
        let query = user.transactions().expect("transactions query failed");
        let index_header = query
            .headers
            .iter()
            .find(|(name, ..)| name == "NO")
            .expect("NO header should exist");

        // Assert
        // 12 rows -> "12" is 2 digits, so width should be max(2, len("NO")) = 2.
        assert_eq!(index_header.1, 2);
    }

    #[test]
    fn refresh_width_count_column_uses_header_length_not_actual_counts() {
        // Arrange
        // FieldVariant::Count width is derived from each row's `link_index` (the third
        // Entry tuple field), not from the item's own `count` field. For Group/Category/
        // Fund/Currency queries that third field is always None (only Transaction queries
        // populate it via refresh_link), so opt_max is always None here and the column
        // width always falls back to the header's own length, regardless of how large the
        // actual `count` values get.
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "B", 200, (1, 1, 2026), "Personal", "Food", "Cash");

        // Act
        let query = user.categories().expect("categories query failed");
        let count_header = query
            .headers
            .iter()
            .find(|(name, ..)| name == "COUNT")
            .expect("COUNT header should exist");
        let food = query
            .rows
            .iter()
            .find(|(item, ..)| item.name() == "Food")
            .expect("Food category should exist");

        // Assert
        assert_eq!(food.0.count, 2); // the actual stored count value
        assert_eq!(count_header.1, "COUNT".len()); // but the column width ignores it
    }

    // endregion

    // region: sort_reverse & sort_clear

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
        assert_eq!(reversed.rows.len(), 1); // the previously-rejected "Transport" row
    }

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

    #[test]
    fn sort_by_name_orders_groups_lexically() {
        // Arrange
        let user = seeded_user();
        user.add_group("Zulu").expect("add_group failed");
        user.add_group("Alpha").expect("add_group failed");

        // Act
        let sorted = user.groups().expect("groups query failed").sort_by_name();

        // Assert
        // 3 groups total: Personal, Zulu, Alpha -> sorted lexically: Alpha, Personal, Zulu
        let names: Vec<&str> = sorted.rows.iter().map(|(item, ..)| item.name()).collect();
        assert_eq!(names, vec!["Alpha", "Personal", "Zulu"]);
    }

    // endregion

    // region: Category::sort_by_type

    #[test]
    fn sort_by_type_orders_single_before_paired() {
        // Arrange
        // seeded_user() registers categories in this order: Food (Single), Transport
        // (Single), Transfer (Paired) -- already Single-before-Paired, so reverse first
        // to confirm sort_by_type actually re-establishes that order rather than it
        // being a coincidence of insertion order.
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
        // The row originally inserted second (index 2, "Earlier") should now sort first.
        assert_eq!(sorted.rows[0].1, 2);
    }

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
        // -200 should sort before 500, so the second-added row (index 2) comes first.
        assert_eq!(sorted.rows[0].1, 2);
    }

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
        // "Large" (inserted first, index 1) has abs amount 200.
        // "Small" (inserted second, index 2) has abs amount 50.
        // abs(-50)=50 sorts before abs(200)=200, so the order should flip.
        let sorted = query.sort_by_abs_amount();

        // Assert
        assert_eq!(sorted.rows[0].1, 2); // "Small" (abs=50) sorts first
        assert_eq!(sorted.rows[1].1, 1); // "Large" (abs=200) sorts last
    }

    #[test]
    fn sort_by_flow_groups_outgoing_before_incoming_when_inserted_in_reverse() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "Out", -500, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "In", 500, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        // sort_by_key(|t| t.amount.value < 0) -> false (incoming) sorts before true (outgoing).
        // "Out" was inserted first (index 1) but is outgoing (key=true), so it should move
        // after "In", which was inserted second (index 2) but is incoming (key=false).
        let sorted = query.sort_by_flow();

        // Assert
        assert_eq!(sorted.rows[0].1, 2); // "In" (incoming, key=false) sorts first
        assert_eq!(sorted.rows[1].1, 1); // "Out" (outgoing, key=true) sorts last
    }

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
        // "Business" < "Personal" lexically, so the second-added row (index 2) sorts first.
        assert_eq!(sorted.rows[0].1, 2);
    }

    #[test]
    fn sort_by_fund_orders_lexically_by_fund_name() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        add_tx(&user, "B", 100, (1, 1, 2026), "Personal", "Food", "Bank");
        let query = user.transactions().expect("transactions query failed");

        // Act
        // "A" (inserted first, index 1) uses "Cash"; "B" (inserted second, index 2) uses "Bank".
        // "Bank" < "Cash" lexically, so the order should flip.
        let sorted = query.sort_by_fund();

        // Assert
        assert_eq!(sorted.rows[0].1, 2); // "B" ("Bank") sorts first
        assert_eq!(sorted.rows[1].1, 1); // "A" ("Cash") sorts last
    }

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
        // "Food" < "Transport" lexically, so the second-added row (index 2) sorts first.
        assert_eq!(sorted.rows[0].1, 2);
    }

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
        // "EUR" < "USD" lexically (Label::fmt uppercases only the first letter of each
        // word and leaves the rest unchanged, so "EUR"/"USD" stay as-is), so the second-added row sorts first.
        assert_eq!(sorted.rows[0].1, 2);
    }

    // endregion

    // region: Transaction filter_*

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
        // "Before" (1 Jan 2026) excluded, "Inside" (15 Jun 2026) included,
        // "OnEdge" (1 Jan 2027) excluded since the upper bound is exclusive.
        assert_eq!(filtered.rows.len(), 1);
    }

    // endregion

    // region: get_row & get_item (via delete/edit error paths)

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

    #[test]
    fn edit_date_rejects_an_invalid_date() {
        // Arrange
        let user = seeded_user();
        add_tx(&user, "A", 100, (1, 1, 2026), "Personal", "Food", "Cash");
        let query = user.transactions().expect("transactions query failed");

        // Act
        let result = query.edit_date(1, 31, 4, 2026); // April has 30 days

        // Assert
        assert!(matches!(result, Err(UserError::Date(_))));
    }

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
        // "Food" is a Single-variant category; this transaction is part of a paired entry.
        let result = query.edit_category(1, "Food");

        // Assert
        assert!(matches!(
            result,
            Err(UserError::Input(InputError::WrongVariant(_)))
        ));
    }

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
