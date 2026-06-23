// Imports from internal user module
use crate::user::{Amount, Category, Date, Flattenable, Fund, Group, HasLabel, Label};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

/// Represents a financial transaction entry within the accounting system.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Transaction {
    /// Combined key descriptor caching specific identity strings.
    pub label: Label,
    /// Absolute atomic balance quantity tracking entity.
    pub amount: Amount,
    /// Gregorian calendar verification date.
    pub date: Date,
    /// Associated group classification tracking entity.
    pub group: Group,
    /// Associated categorisation validation tracking rules.
    pub category: Category,
    /// Designated account allocation target asset node.
    pub fund: Fund,
    /// Optional pairing key referencing secondary entries.
    pub link: Option<Uuid>,
}

impl Transaction {
    /// Maps a single SQLite row to a `Transaction` instance using explicit column offsets.
    ///
    /// The expected layout of the query results must match the comment annotations below.
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Transaction {
            label: Label::from_row_offset(row, 0)?, // Columns: 0 (id), 1 (name), 2 (description)
            amount: Amount::from_row_offset(row, 3)?, // Columns: 3 (value), 4 (currency id), 5 (currency name), 6 (currency count)
            date: Date::from_row_offset(row, 7)?,     // Columns: 7 (day), 8 (month), 9 (year)
            group: Group::from_row_offset(row, 10)?, // Columns: 10 (id), 11 (name), 12 (group count)
            category: Category::from_row_offset(row, 13)?, // Columns: 13 (id), 14 (name), 15 (variant), 16 (category count)
            fund: Fund::from_row_offset(row, 17)?, // Columns: 17 (id), 18 (name), 19 (fund count)
            link: row.get(20)?,                    // Column:  20 (linked transaction id)
        })
    }
}

impl Display for Transaction {
    /// Formats the transaction data as a single-line pipe-separated string.
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {} | {}",
            self.label, self.amount, self.date, self.group, self.category, self.fund
        )
    }
}

impl HasLabel for Transaction {
    /// Returns a reference to the underlying structural `Label`.
    fn label(&self) -> &Label {
        &self.label
    }

    /// Declares the corresponding database source table identifier string.
    fn table() -> &'static str {
        "transactions"
    }
}

impl Flattenable for Transaction {
    /// Flattens categorical tracking vectors into raw field vector elements.
    fn flatten(&self) -> Vec<String> {
        let mut flat = self.label.flatten();
        flat.push(self.amount.to_string());
        flat.push(self.date.to_string());
        flat.push(self.group.to_string());
        flat.push(self.category.to_string());
        flat.push(self.fund.to_string());
        flat
    }
}

// region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // region: helpers

    /// Builds an in-memory connection seeded with a single transaction-shaped row.
    fn connection_with_transaction_row() -> (Connection, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE t (
                id BLOB, name TEXT, description TEXT,
                value INTEGER, cur_id BLOB, cur_name TEXT, cur_count INTEGER,
                day INTEGER, month INTEGER, year INTEGER,
                grp_id BLOB, grp_name TEXT, grp_count INTEGER,
                cat_id BLOB, cat_name TEXT, cat_variant INTEGER, cat_count INTEGER,
                fund_id BLOB, fund_name TEXT, fund_count INTEGER,
                link_id BLOB
            );",
            (),
        )
        .expect("failed to create table");

        let tx_id = Uuid::new_v4();
        let cur_id = Uuid::new_v4();
        let grp_id = Uuid::new_v4();
        let cat_id = Uuid::new_v4();
        let fund_id = Uuid::new_v4();
        let link_id = Uuid::new_v4();

        conn.execute(
            "INSERT INTO t (
                id, name, description,
                value, cur_id, cur_name, cur_count,
                day, month, year,
                grp_id, grp_name, grp_count,
                cat_id, cat_name, cat_variant, cat_count,
                fund_id, fund_name, fund_count,
                link_id
            ) VALUES (
                ?1, ?2, ?3,
                ?4, ?5, ?6, ?7,
                ?8, ?9, ?10,
                ?11, ?12, ?13,
                ?14, ?15, ?16, ?17,
                ?18, ?19, ?20,
                ?21
            )",
            rusqlite::params![
                tx_id,
                "Groceries",
                "Weekly shop",
                1050i64,
                cur_id,
                "US Dollar",
                2i64,
                15u8,
                6u8,
                2026u16,
                grp_id,
                "Personal",
                4i64,
                cat_id,
                "Food",
                0i64, // CategoryVariant::Single
                1i64,
                fund_id,
                "Cash",
                3i64,
                link_id,
            ],
        )
        .expect("failed to insert row");

        (conn, tx_id, cur_id, grp_id, cat_id, fund_id, link_id)
    }

    // endregion

    // region: Transaction::from_row

    /// Verifies that standard row deserialization maps target identity metadata, value totals, and date blocks cleanly.
    #[test]
    fn from_row_maps_label_amount_and_date_fields() {
        // Arrange
        let (conn, tx_id, cur_id, ..) = connection_with_transaction_row();

        // Act
        let tx: Transaction = conn
            .query_row("SELECT * FROM t", (), Transaction::from_row)
            .expect("query should succeed");

        // Assert
        assert_eq!(tx.label.id, tx_id);
        assert_eq!(tx.label.name, "Groceries");
        assert_eq!(tx.label.description, Some(String::from("Weekly shop")));
        assert_eq!(tx.amount.value, 1050);
        assert_eq!(tx.amount.currency.label.id, cur_id);
        assert_eq!(tx.amount.currency.label.name, "US Dollar");
        assert_eq!(tx.amount.currency.count, 2);
        assert_eq!((tx.date.day, tx.date.month, tx.date.year), (15, 6, 2026));
    }

    /// Verifies row extraction processes tracking structures accurately given sequential group constraints.
    #[test]
    fn from_row_maps_group_with_its_count_column() {
        // Arrange
        let (conn, _, _, grp_id, ..) = connection_with_transaction_row();

        // Act
        let tx: Transaction = conn
            .query_row("SELECT * FROM t", (), Transaction::from_row)
            .expect("query should succeed");

        // Assert
        assert_eq!(tx.group.label.id, grp_id);
        assert_eq!(tx.group.label.name, "Personal");
        assert_eq!(tx.group.count, 4);
    }

    /// Verifies row mapping decodes sequence targets into valid localized categorization bounds.
    #[test]
    fn from_row_maps_category_with_variant_and_count_columns() {
        // Arrange
        let (conn, .., cat_id, _, _) = connection_with_transaction_row();

        // Act
        let tx: Transaction = conn
            .query_row("SELECT * FROM t", (), Transaction::from_row)
            .expect("query should succeed");

        // Assert
        assert_eq!(tx.category.label.id, cat_id);
        assert_eq!(tx.category.label.name, "Food");
        assert_eq!(tx.category.variant, crate::user::CategoryVariant::Single);
        assert_eq!(tx.category.count, 1);
    }

    /// Verifies row translation maps associated allocation targets safely from basic engine layers.
    #[test]
    fn from_row_maps_fund_with_its_count_column() {
        // Arrange
        let (conn, .., fund_id, _) = connection_with_transaction_row();

        // Act
        let tx: Transaction = conn
            .query_row("SELECT * FROM t", (), Transaction::from_row)
            .expect("query should succeed");

        // Assert
        assert_eq!(tx.fund.label.id, fund_id);
        assert_eq!(tx.fund.label.name, "Cash");
        assert_eq!(tx.fund.count, 3);
    }

    /// Verifies that explicit cross-link references map smoothly to target structural parameters.
    #[test]
    fn from_row_maps_link_id_as_the_final_column() {
        // Arrange
        let (conn, .., link_id) = connection_with_transaction_row();

        // Act
        let tx: Transaction = conn
            .query_row("SELECT * FROM t", (), Transaction::from_row)
            .expect("query should succeed");

        // Assert
        assert_eq!(tx.link, Some(link_id));
    }

    /// Verifies that empty record configurations fallback gracefully to default optional types.
    #[test]
    fn from_row_maps_null_link_as_none() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE t (
                id BLOB, name TEXT, description TEXT,
                value INTEGER, cur_id BLOB, cur_name TEXT, cur_count INTEGER,
                day INTEGER, month INTEGER, year INTEGER,
                grp_id BLOB, grp_name TEXT, grp_count INTEGER,
                cat_id BLOB, cat_name TEXT, cat_variant INTEGER, cat_count INTEGER,
                fund_id BLOB, fund_name TEXT, fund_count INTEGER,
                link_id BLOB
            );",
            (),
        )
        .expect("failed to create table");
        conn.execute(
            "INSERT INTO t (
                id, name, description,
                value, cur_id, cur_name, cur_count,
                day, month, year,
                grp_id, grp_name, grp_count,
                cat_id, cat_name, cat_variant, cat_count,
                fund_id, fund_name, fund_count,
                link_id
            ) VALUES (
                ?1, ?2, NULL,
                ?3, ?4, ?5, ?6,
                ?7, ?8, ?9,
                ?10, ?11, ?12,
                ?13, ?14, ?15, ?16,
                ?17, ?18, ?19,
                NULL
            )",
            rusqlite::params![
                Uuid::new_v4(),
                "Standalone",
                500i64,
                Uuid::new_v4(),
                "US Dollar",
                0i64,
                1u8,
                1u8,
                2026u16,
                Uuid::new_v4(),
                "Personal",
                0i64,
                Uuid::new_v4(),
                "Food",
                0i64,
                0i64,
                Uuid::new_v4(),
                "Cash",
                0i64,
            ],
        )
        .expect("failed to insert row");

        // Act
        let tx: Transaction = conn
            .query_row("SELECT * FROM t", (), Transaction::from_row)
            .expect("query should succeed");

        // Assert
        assert_eq!(tx.link, None);
    }

    // endregion

    // region: HasLabel for Transaction

    /// Verifies trait signatures route accurately to internal label properties.
    #[test]
    fn has_label_label_returns_the_underlying_label() {
        // Arrange
        let (conn, tx_id, ..) = connection_with_transaction_row();
        let tx: Transaction = conn
            .query_row("SELECT * FROM t", (), Transaction::from_row)
            .expect("query should succeed");

        // Act
        let label_ref = tx.label();

        // Assert
        assert_eq!(label_ref.id, tx_id);
    }

    /// Verifies trait mappings match the designated collection target labels.
    #[test]
    fn has_label_table_returns_transactions() {
        // Arrange & Act
        let table = Transaction::table();

        // Assert
        assert_eq!(table, "transactions");
    }

    // endregion
}

// endregion
