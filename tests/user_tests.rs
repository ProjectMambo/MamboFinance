use mambofinance::user::User;

fn setup() -> User {
    User::new_in_memory("test").unwrap()
}

// ===== GROUP =====

#[test]
fn add_group() {
    let user = setup();
    user.add_group("Food").unwrap();
}

#[test]
fn add_duplicate_group_is_ok() {
    let user = setup();
    user.add_group("Food").unwrap();
    user.add_group("Food").unwrap(); // should not error
}

#[test]
fn add_multiple_groups() {
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
fn add_duplicate_category_is_ok() {
    let user = setup();
    user.add_category("Bills").unwrap();
    user.add_category("Bills").unwrap(); // silently ok
}

// ===== FUND =====

#[test]
fn add_fund() {
    let user = setup();
    user.add_fund("Checking").unwrap();
}

#[test]
fn add_duplicate_fund_is_ok() {
    let user = setup();
    user.add_fund("Checking").unwrap();
    user.add_fund("Checking").unwrap(); // silently ok
}

// ===== CURRENCY =====

#[test]
fn add_currency() {
    let user = setup();
    user.add_currency("MYR").unwrap();
}

#[test]
fn add_duplicate_currency_is_ok() {
    let user = setup();
    user.add_currency("MYR").unwrap();
    user.add_currency("MYR").unwrap(); // silently ok
}

// ===== TRANSACTION =====

fn setup_transaction(user: &User) {
    user.add_currency("MYR")
        .unwrap()
        .add_group("Food")
        .unwrap()
        .add_category("Groceries")
        .unwrap()
        .add_fund("Checking")
        .unwrap();
}

#[test]
fn add_transaction() {
    let user = setup();
    setup_transaction(&user);
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
    setup_transaction(&user);
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
    setup_transaction(&user);
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
    setup_transaction(&user);
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
    setup_transaction(&user);
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
    setup_transaction(&user);
    // "Groceries" is Single variant, but add_paired_category would be Paired
    // add_transaction requires Single variant category
    user.add_paired_category("PairedCat").unwrap();
    assert!(
        user.add_transaction(
            "Lunch",
            None,
            (1500, "MYR"),
            (1, 6, 2026),
            "Food",
            "PairedCat",
            "Checking", // ← Paired category in single transaction
        )
        .is_err()
    );
}

#[test]
fn add_transaction_invalid_date_errors() {
    let user = setup();
    setup_transaction(&user);
    assert!(
        user.add_transaction(
            "Lunch",
            None,
            (1500, "MYR"),
            (32, 13, 2026), // ← invalid day and month
            "Food",
            "Groceries",
            "Checking",
        )
        .is_err()
    );
}

#[test]
fn add_multiple_transactions() {
    let user = setup();
    setup_transaction(&user);
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

fn setup_paired(user: &User) {
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

#[test]
fn add_paired_transaction() {
    let user = setup();
    setup_paired(&user);
    user.add_paired_transaction(
        "Move funds",
        None,
        (50000, "MYR"), // source
        (50000, "MYR"), // target
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
    setup_paired(&user);
    user.add_currency("USD").unwrap();
    user.add_paired_transaction(
        "Exchange",
        None,
        (50000, "MYR"), // source
        (10000, "USD"), // target — different currency and amount
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
    setup_paired(&user);
    user.add_category("Single").unwrap();
    assert!(
        user.add_paired_transaction(
            "Move funds",
            None,
            (50000, "MYR"),
            (50000, "MYR"),
            (1, 6, 2026),
            "Transfer",
            "Single", // ← Single category in paired transaction
            "Checking",
            "Savings",
        )
        .is_err()
    );
}

#[test]
fn add_paired_transaction_unknown_source_fund_errors() {
    let user = setup();
    setup_paired(&user);
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
    setup_paired(&user);
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

// ===== CHAINING =====

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
