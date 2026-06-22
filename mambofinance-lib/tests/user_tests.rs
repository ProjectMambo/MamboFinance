/* // mambofinance-lib/tests/user_tests.rs
//
// Integration tests against User's public API only.
// Private methods (ls_*, get_*, check_category_variant) are tested
// internally in src/user.rs's #[cfg(test)] block instead.

use mambofinance_lib::user::User;

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

// ===== GROUP =====

#[test]
fn add_group() {
    let user = setup();
    user.add_group("Food").unwrap();
}

#[test]
fn add_duplicate_group_is_idempotent() {
    let user = setup();
    user.add_group("Food").unwrap();
    user.add_group("Food").unwrap();
}

#[test]
fn add_multiple_groups_chained() {
    let user = setup();
    user.add_group("Food")
        .unwrap()
        .add_group("Housing")
        .unwrap()
        .add_group("Transport")
        .unwrap();
}

// ===== CATEGORY =====

#[test]
fn add_category() {
    let user = setup();
    user.add_category("Groceries").unwrap();
}

#[test]
fn add_paired_category() {
    let user = setup();
    user.add_paired_category("Transfer").unwrap();
}

#[test]
fn add_duplicate_category_is_idempotent() {
    let user = setup();
    user.add_category("Bills").unwrap();
    user.add_category("Bills").unwrap();
}

// ===== FUND =====

#[test]
fn add_fund() {
    let user = setup();
    user.add_fund("Checking").unwrap();
}

#[test]
fn add_duplicate_fund_is_idempotent() {
    let user = setup();
    user.add_fund("Checking").unwrap();
    user.add_fund("Checking").unwrap();
}

// ===== CURRENCY =====

#[test]
fn add_currency() {
    let user = setup();
    user.add_currency("MYR").unwrap();
}

#[test]
fn add_duplicate_currency_is_idempotent() {
    let user = setup();
    user.add_currency("MYR").unwrap();
    user.add_currency("MYR").unwrap();
}

// ===== TRANSACTION =====

#[test]
fn add_transaction() {
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
}

#[test]
fn add_transaction_with_description() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_transaction(
        "Lunch",
        Some("KFC at mall"),
        (1500, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();
}

#[test]
fn add_transaction_unknown_group_errors() {
    let user = setup();
    setup_transaction_deps(&user);
    assert!(
        user.add_transaction(
            "Lunch",
            None,
            (1500, "MYR"),
            (1, 6, 2026),
            "UNKNOWN",
            "Groceries",
            "Checking",
        )
        .is_err()
    );
}

#[test]
fn add_transaction_unknown_fund_errors() {
    let user = setup();
    setup_transaction_deps(&user);
    assert!(
        user.add_transaction(
            "Lunch",
            None,
            (1500, "MYR"),
            (1, 6, 2026),
            "Food",
            "Groceries",
            "UNKNOWN",
        )
        .is_err()
    );
}

#[test]
fn add_transaction_unknown_currency_errors() {
    let user = setup();
    setup_transaction_deps(&user);
    assert!(
        user.add_transaction(
            "Lunch",
            None,
            (1500, "UNKNOWN"),
            (1, 6, 2026),
            "Food",
            "Groceries",
            "Checking",
        )
        .is_err()
    );
}

#[test]
fn add_transaction_wrong_category_variant_errors() {
    let user = setup();
    setup_transaction_deps(&user);
    user.add_paired_category("PairedCat").unwrap();
    assert!(
        user.add_transaction(
            "Lunch",
            None,
            (1500, "MYR"),
            (1, 6, 2026),
            "Food",
            "PairedCat",
            "Checking",
        )
        .is_err()
    );
}

#[test]
fn add_transaction_invalid_date_errors() {
    let user = setup();
    setup_transaction_deps(&user);
    assert!(
        user.add_transaction(
            "Lunch",
            None,
            (1500, "MYR"),
            (32, 13, 2026),
            "Food",
            "Groceries",
            "Checking",
        )
        .is_err()
    );
}

#[test]
fn add_multiple_transactions_chained() {
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
    .unwrap()
    .add_transaction(
        "Dinner",
        None,
        (2500, "MYR"),
        (1, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap()
    .add_transaction(
        "Breakfast",
        None,
        (800, "MYR"),
        (2, 6, 2026),
        "Food",
        "Groceries",
        "Checking",
    )
    .unwrap();
}

// ===== PAIRED TRANSACTION =====

#[test]
fn add_paired_transaction() {
    let user = setup();
    setup_paired_deps(&user);
    user.add_paired_transaction(
        "Move funds",
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
}

#[test]
fn add_paired_transaction_different_currencies() {
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
}

#[test]
fn add_paired_transaction_wrong_category_errors() {
    let user = setup();
    setup_paired_deps(&user);
    user.add_category("Single").unwrap();
    assert!(
        user.add_paired_transaction(
            "Move funds",
            None,
            (50000, "MYR"),
            (50000, "MYR"),
            (1, 6, 2026),
            "Transfer",
            "Single",
            "Checking",
            "Savings",
        )
        .is_err()
    );
}

#[test]
fn add_paired_transaction_unknown_source_fund_errors() {
    let user = setup();
    setup_paired_deps(&user);
    assert!(
        user.add_paired_transaction(
            "Move funds",
            None,
            (50000, "MYR"),
            (50000, "MYR"),
            (1, 6, 2026),
            "Transfer",
            "Internal",
            "UNKNOWN",
            "Savings",
        )
        .is_err()
    );
}

#[test]
fn add_paired_transaction_unknown_target_fund_errors() {
    let user = setup();
    setup_paired_deps(&user);
    assert!(
        user.add_paired_transaction(
            "Move funds",
            None,
            (50000, "MYR"),
            (50000, "MYR"),
            (1, 6, 2026),
            "Transfer",
            "Internal",
            "Checking",
            "UNKNOWN",
        )
        .is_err()
    );
}

// ===== QUERY ENTRY POINTS =====

#[test]
fn transactions_entry_point() {
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
    assert!(user.transactions().is_ok());
}

#[test]
fn groups_entry_point() {
    let user = setup();
    user.add_group("Food").unwrap();
    assert!(user.groups().is_ok());
}

#[test]
fn categories_entry_point() {
    let user = setup();
    user.add_category("Groceries").unwrap();
    assert!(user.categories().is_ok());
}

#[test]
fn funds_entry_point() {
    let user = setup();
    user.add_fund("Checking").unwrap();
    assert!(user.funds().is_ok());
}

#[test]
fn currencies_entry_point() {
    let user = setup();
    user.add_currency("MYR").unwrap();
    assert!(user.currencies().is_ok());
}

// ===== FULL CHAIN =====

#[test]
fn full_chain() {
    let user = setup();
    user.add_currency("MYR")
        .unwrap()
        .add_group("Food")
        .unwrap()
        .add_group("Transfer")
        .unwrap()
        .add_category("Groceries")
        .unwrap()
        .add_paired_category("Internal")
        .unwrap()
        .add_fund("Checking")
        .unwrap()
        .add_fund("Savings")
        .unwrap()
        .add_transaction(
            "Lunch",
            None,
            (1500, "MYR"),
            (1, 6, 2026),
            "Food",
            "Groceries",
            "Checking",
        )
        .unwrap()
        .add_paired_transaction(
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
}
 */