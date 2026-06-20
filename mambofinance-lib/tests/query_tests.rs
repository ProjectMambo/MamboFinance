// mambofinance-lib/tests/query_tests.rs
//
// Integration tests against Query<T>'s public API only.
// Private methods (get_by_index, delete_by_id, edit_unique_by_id,
// edit_by_id, edit_shared_field) are tested internally in
// src/user/query.rs's #[cfg(test)] block instead.

use mambofinance_lib::user::{CategoryVariant, User};

fn setup() -> User {
    User::new_in_memory("test").unwrap()
}

fn setup_transaction_deps(user: &User) {
    user.add_currency("MYR")
        .unwrap()
        .add_group("Food")
        .unwrap()
        .add_category("Groceries")
        .unwrap()
        .add_fund("Checking")
        .unwrap();
}

fn setup_paired_deps(user: &User) {
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
}

// ===== sort_by_name (generic, HasLabel) =====

#[test]
fn sort_by_name_orders_groups_lexically() {
    let user = setup();
    user.add_group("Zebra").unwrap();
    user.add_group("Apple").unwrap();

    let query = user.groups().unwrap().sort_by_name();
    // names go through Label::fmt, so case should already be normalized
    assert!(query.rows[0].label.name <= query.rows[1].label.name);
}

#[test]
fn sort_by_name_orders_funds_lexically() {
    let user = setup();
    user.add_fund("Savings").unwrap();
    user.add_fund("Checking").unwrap();

    let query = user.funds().unwrap().sort_by_name();
    assert_eq!(query.rows[0].label.name, "Checking");
    assert_eq!(query.rows[1].label.name, "Savings");
}

// ===== Category: sort_by_type, edit_name, edit_variant, force_edit_variant =====

#[test]
fn category_sort_by_type_groups_single_before_paired() {
    let user = setup();
    user.add_paired_category("Transfer").unwrap();
    user.add_category("Groceries").unwrap();

    let query = user.categories().unwrap().sort_by_type();
    assert_eq!(query.rows[0].variant, CategoryVariant::Single);
    assert_eq!(query.rows[1].variant, CategoryVariant::Paired);
}

#[test]
fn category_edit_name() {
    let user = setup();
    user.add_category("Groceries").unwrap();
    user.categories()
        .unwrap()
        .edit_name(1, "Food Shopping")
        .unwrap();

    let refreshed = user.categories().unwrap();
    assert_eq!(refreshed.rows[0].label.name, "Food Shopping");
}

#[test]
fn category_edit_name_collision_errors() {
    let user = setup();
    user.add_category("Groceries").unwrap();
    user.add_category("Bills").unwrap();

    let query = user.categories().unwrap().sort_by_name();
    let first_name = query.rows[0].label.name.clone();
    let second_name = query.rows[1].label.name.clone();

    // renaming row 1 to row 2's existing name should fail
    let result = query.edit_name(1, &second_name);
    assert!(result.is_err());
    let _ = first_name; // silence unused warning if not asserted further
}

#[test]
fn category_edit_variant_succeeds_when_unused() {
    let user = setup();
    user.add_category("Groceries").unwrap();

    user.categories()
        .unwrap()
        .edit_variant(1, CategoryVariant::Paired)
        .unwrap();

    let refreshed = user.categories().unwrap();
    assert_eq!(refreshed.rows[0].variant, CategoryVariant::Paired);
}

#[test]
fn category_edit_variant_blocked_when_in_use() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_transaction(
        "Lunch",
        None,
        (1500, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();

    let result = user
        .categories()
        .unwrap()
        .edit_variant(1, CategoryVariant::Paired);
    assert!(result.is_err());
}

#[test]
fn category_force_edit_variant_unlinks_transactions() {
    let user = setup();
    setup_paired_deps(&user);
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

    let categories = user.categories().unwrap();
    let idx = categories
        .rows
        .iter()
        .position(|c| c.label.name == "Internal")
        .unwrap()
        + 1;

    user.categories()
        .unwrap()
        .force_edit_variant(idx, CategoryVariant::Single)
        .unwrap();

    let refreshed = user.categories().unwrap();
    let internal = refreshed
        .rows
        .iter()
        .find(|c| c.label.name == "Internal")
        .unwrap();
    assert_eq!(internal.variant, CategoryVariant::Single);
}

// ===== Transaction: sort_by_*, filter_*, edit_*, delete (paired-aware) =====

#[test]
fn transaction_sort_by_date() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_transaction(
        "Later",
        None,
        (100, "MYR"),
        (15, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();
    user.add_transaction(
        "Earlier",
        None,
        (100, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();

    let query = user.transactions().unwrap().sort_by_date();
    assert_eq!(query.rows[0].label.name, "Earlier");
    assert_eq!(query.rows[1].label.name, "Later");
}

#[test]
fn transaction_sort_by_amount() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_transaction(
        "Big",
        None,
        (5000, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();
    user.add_transaction(
        "Small",
        None,
        (100, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();

    let query = user.transactions().unwrap().sort_by_amount();
    assert_eq!(query.rows[0].label.name, "Small");
    assert_eq!(query.rows[1].label.name, "Big");
}

#[test]
fn transaction_sort_by_abs_amount_treats_negative_and_positive_by_magnitude() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_transaction(
        "Neg",
        None,
        (-5000, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();
    user.add_transaction(
        "Pos",
        None,
        (100, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();

    let query = user.transactions().unwrap().sort_by_abs_amount();
    assert_eq!(query.rows[0].label.name, "Pos"); // |100| < |-5000|
    assert_eq!(query.rows[1].label.name, "Neg");
}

#[test]
fn transaction_sort_by_currency() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_currency("USD").unwrap();
    user.add_transaction(
        "USD Txn",
        None,
        (100, "USD"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();
    user.add_transaction(
        "MYR Txn",
        None,
        (100, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();

    let query = user.transactions().unwrap().sort_by_currency();
    assert_eq!(query.rows[0].amount.currency.label.name, "MYR");
    assert_eq!(query.rows[1].amount.currency.label.name, "USD");
}

#[test]
fn transaction_filter_group_matches_only_intended() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_group("Transport").unwrap();
    user.add_category("Petrol").unwrap();
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
    user.add_transaction(
        "Grab",
        None,
        (100, "MYR"),
        (1, 6, 2026),
        "Transport",
        "Petrol",
        "Checking",
    )
    .unwrap();

    let query = user.transactions().unwrap().filter_group("Food");
    assert_eq!(query.rows.len(), 1);
    assert_eq!(query.rows[0].label.name, "Lunch");
}

#[test]
fn transaction_filter_category_does_not_match_group_name() {
    // Regression: filter_category must compare category.label.name, not group.label.name.
    let user = setup();
    setup_transaction_deps(&user);
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

    let by_category = user.transactions().unwrap().filter_category("Groceries");
    assert_eq!(by_category.rows.len(), 1);

    let by_group_name_as_category = user.transactions().unwrap().filter_category("Food");
    assert_eq!(by_group_name_as_category.rows.len(), 0);
}

#[test]
fn transaction_filter_fund() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_fund("Savings").unwrap();
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
    user.add_transaction(
        "Deposit",
        None,
        (100, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Savings",
    )
    .unwrap();

    let query = user.transactions().unwrap().filter_fund("Savings");
    assert_eq!(query.rows.len(), 1);
    assert_eq!(query.rows[0].label.name, "Deposit");
}

#[test]
fn transaction_filter_currency() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_currency("USD").unwrap();
    user.add_transaction(
        "MYR Txn",
        None,
        (100, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();
    user.add_transaction(
        "USD Txn",
        None,
        (100, "USD"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();

    let query = user.transactions().unwrap().filter_currency("USD");
    assert_eq!(query.rows.len(), 1);
    assert_eq!(query.rows[0].label.name, "USD Txn");
}

#[test]
fn transaction_chained_filter_then_sort() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_transaction(
        "B",
        None,
        (200, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();
    user.add_transaction(
        "A",
        None,
        (100, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();

    let query = user
        .transactions()
        .unwrap()
        .filter_group("Food")
        .sort_by_amount();
    assert_eq!(query.rows.len(), 2);
    assert_eq!(query.rows[0].label.name, "A");
}

#[test]
fn transaction_edit_name() {
    let user = setup();
    setup_transaction_deps(&user);
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

    user.transactions().unwrap().edit_name(1, "Dinner").unwrap();
    let refreshed = user.transactions().unwrap();
    assert_eq!(refreshed.rows[0].label.name, "Dinner");
}

#[test]
fn transaction_edit_name_updates_both_paired_sides() {
    let user = setup();
    setup_paired_deps(&user);
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

    user.transactions()
        .unwrap()
        .edit_name(1, "Relocated")
        .unwrap();
    let refreshed = user.transactions().unwrap();
    assert!(refreshed.rows.iter().all(|t| t.label.name == "Relocated"));
}

#[test]
fn transaction_edit_amount_only_affects_single_row() {
    let user = setup();
    setup_paired_deps(&user);
    user.add_currency("USD").unwrap();
    user.add_paired_transaction(
        "Exchange",
        None,
        (50000, "MYR"),
        (10000, "USD"),
        (1, 6, 2026),
        "Transfer",
        "Internal",
        "Checking",
        "Savings",
    )
    .unwrap();

    let query = user.transactions().unwrap().sort_by_currency(); // MYR first
    let myr_no = query
        .rows
        .iter()
        .position(|t| t.amount.currency.label.name == "MYR")
        .unwrap()
        + 1;

    user.transactions()
        .unwrap()
        .sort_by_currency()
        .edit_amount(myr_no, 99999)
        .unwrap();

    let refreshed = user.transactions().unwrap();
    let myr_leg = refreshed
        .rows
        .iter()
        .find(|t| t.amount.currency.label.name == "MYR")
        .unwrap();
    let usd_leg = refreshed
        .rows
        .iter()
        .find(|t| t.amount.currency.label.name == "USD")
        .unwrap();
    assert_eq!(myr_leg.amount.value, 99999);
    assert_eq!(usd_leg.amount.value, 10000); // untouched
}

#[test]
fn transaction_edit_category_allows_same_variant() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_category("OtherSingle").unwrap();
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

    let result = user.transactions().unwrap().edit_category(1, "OtherSingle");
    assert!(result.is_ok());
    let refreshed = user.transactions().unwrap();
    assert_eq!(refreshed.rows[0].category.label.name, "OtherSingle");
}

#[test]
fn transaction_edit_category_blocks_variant_mismatch() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_paired_category("SomePaired").unwrap();
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

    let result = user.transactions().unwrap().edit_category(1, "SomePaired");
    assert!(result.is_err());
}

#[test]
fn transaction_edit_fund() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_fund("Savings").unwrap();
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

    user.transactions()
        .unwrap()
        .edit_fund(1, "Savings")
        .unwrap();
    let refreshed = user.transactions().unwrap();
    assert_eq!(refreshed.rows[0].fund.label.name, "Savings");
}

#[test]
fn transaction_edit_currency() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_currency("USD").unwrap();
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

    user.transactions()
        .unwrap()
        .edit_currency(1, "USD")
        .unwrap();
    let refreshed = user.transactions().unwrap();
    assert_eq!(refreshed.rows[0].amount.currency.label.name, "USD");
}

#[test]
fn transaction_edit_group() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_group("Transport").unwrap();
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

    user.transactions()
        .unwrap()
        .edit_group(1, "Transport")
        .unwrap();
    let refreshed = user.transactions().unwrap();
    assert_eq!(refreshed.rows[0].group.label.name, "Transport");
}

#[test]
fn transaction_edit_group_updates_both_paired_sides() {
    let user = setup();
    setup_paired_deps(&user);
    user.add_group("Other").unwrap();
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

    user.transactions().unwrap().edit_group(1, "Other").unwrap();
    let refreshed = user.transactions().unwrap();
    assert!(refreshed.rows.iter().all(|t| t.group.label.name == "Other"));
}

#[test]
fn transaction_edit_date() {
    let user = setup();
    setup_transaction_deps(&user);
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

    user.transactions()
        .unwrap()
        .edit_date(1, 25, 12, 2026)
        .unwrap();
    let refreshed = user.transactions().unwrap();
    assert_eq!(refreshed.rows[0].date.day, 25);
    assert_eq!(refreshed.rows[0].date.month, 12);
    assert_eq!(refreshed.rows[0].date.year, 2026);
}

#[test]
fn transaction_edit_date_invalid_errors() {
    let user = setup();
    setup_transaction_deps(&user);
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

    let result = user.transactions().unwrap().edit_date(1, 31, 2, 2026); // Feb 31 invalid
    assert!(result.is_err());
}

#[test]
fn transaction_delete_single() {
    let user = setup();
    setup_transaction_deps(&user);
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

    user.transactions().unwrap().delete(1).unwrap();
    let remaining = user.transactions().unwrap();
    assert_eq!(remaining.rows.len(), 0);
}

#[test]
fn transaction_delete_paired_removes_both_sides() {
    let user = setup();
    setup_paired_deps(&user);
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

    let before = user.transactions().unwrap();
    assert_eq!(before.rows.len(), 2);

    user.transactions().unwrap().delete(1).unwrap();
    let after = user.transactions().unwrap();
    assert_eq!(
        after.rows.len(),
        0,
        "deleting one half of a pair must remove both"
    );
}

#[test]
fn transaction_delete_invalid_index_errors() {
    let user = setup();
    setup_transaction_deps(&user);
    let result = user.transactions().unwrap().delete(1);
    assert!(result.is_err());
}

// ===== Group / Fund / Currency: print/delete/edit_name (shared shape) =====

#[test]
fn group_delete() {
    let user = setup();
    user.add_group("Food").unwrap();
    user.groups().unwrap().delete(1).unwrap();
    let remaining = user.groups().unwrap();
    assert_eq!(remaining.rows.len(), 0);
}

#[test]
fn group_edit_name() {
    let user = setup();
    user.add_group("Food").unwrap();
    user.groups()
        .unwrap()
        .edit_name(1, "Groceries Spend")
        .unwrap();
    let refreshed = user.groups().unwrap();
    assert_eq!(refreshed.rows[0].label.name, "Groceries Spend");
}

#[test]
fn fund_delete() {
    let user = setup();
    user.add_fund("Checking").unwrap();
    user.funds().unwrap().delete(1).unwrap();
    let remaining = user.funds().unwrap();
    assert_eq!(remaining.rows.len(), 0);
}

#[test]
fn fund_edit_name() {
    let user = setup();
    user.add_fund("Checking").unwrap();
    user.funds().unwrap().edit_name(1, "Main Checking").unwrap();
    let refreshed = user.funds().unwrap();
    assert_eq!(refreshed.rows[0].label.name, "Main Checking");
}

#[test]
fn currency_delete() {
    let user = setup();
    user.add_currency("MYR").unwrap();
    user.currencies().unwrap().delete(1).unwrap();
    let remaining = user.currencies().unwrap();
    assert_eq!(remaining.rows.len(), 0);
}

#[test]
fn currency_edit_name() {
    let user = setup();
    user.add_currency("MYR").unwrap();
    user.currencies().unwrap().edit_name(1, "Ringgit").unwrap();
    let refreshed = user.currencies().unwrap();
    assert_eq!(refreshed.rows[0].label.name, "Ringgit");
}

// ===== Cascade delete behavior =====

#[test]
fn deleting_group_cascades_to_referencing_transactions() {
    let user = setup();
    setup_transaction_deps(&user);
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

    user.groups().unwrap().delete(1).unwrap(); // delete "Food"
    let remaining = user.transactions().unwrap();
    assert_eq!(
        remaining.rows.len(),
        0,
        "deleting a referenced group should cascade-delete its transactions"
    );
}

#[test]
fn deleting_currency_cascades_to_referencing_transactions() {
    let user = setup();
    setup_transaction_deps(&user);
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

    user.currencies().unwrap().delete(1).unwrap(); // delete "MYR"
    let remaining = user.transactions().unwrap();
    assert_eq!(remaining.rows.len(), 0);
}

// ===== Out-of-range index handling (shared across types) =====

#[test]
fn delete_out_of_range_index_errors() {
    let user = setup();
    user.add_group("Food").unwrap();
    let result = user.groups().unwrap().delete(99);
    assert!(result.is_err());
}

#[test]
fn edit_name_out_of_range_index_errors() {
    let user = setup();
    user.add_group("Food").unwrap();
    let result = user.groups().unwrap().edit_name(99, "New Name");
    assert!(result.is_err());
}
