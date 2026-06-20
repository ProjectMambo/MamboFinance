#![allow(dead_code)]

use crate::user::CategoryVariant;
use crate::user::{User, UserError};

pub fn simulate_user_activity(user: &User) -> Result<(), UserError> {
    // =========================================================================
    // 1. MASTER SEED DATA PROVISIONING
    // =========================================================================
    user.add_currency("USD")?
        .add_currency("EUR")?
        .add_currency("GBP")?
        .add_currency("JPY")?
        .add_currency("MYR")?
        .add_group("Housing & Utilities")?
        .add_group("Food & Dining")?
        .add_group("Transportation")?
        .add_group("Entertainment & Leisure")?
        .add_group("Healthcare")?
        .add_group("Investments")?
        .add_group("Salary & Business Income")?
        .add_group("Education & Self-Improvement")? // Extended group
        .add_group("Miscellanous Mishaps")? // Extended group
        .add_fund("Main Checking")?
        .add_fund("High-Yield Savings")?
        .add_fund("Physical Wallet")?
        .add_fund("Investment Brokerage")?
        .add_fund("Credit Card Balance")?
        .add_category("Groceries")?
        .add_category("Restaurants")?
        .add_category("Rent & Mortgage")?
        .add_category("Electricity Bill")?
        .add_category("Gas & Fuel")?
        .add_category("Public Transit")?
        .add_category("Movies & Streaming")?
        .add_category("Gym Membership")?
        .add_category("Medical Insurance")?
        .add_category("Stocks ETF")?
        .add_category("Primary Paycheck")?
        .add_category("Freelance Gig")?
        .add_category("Books & Courses")? // Extended category
        .add_category("Fines & Fees")? // Extended category
        .add_paired_category("Internal Transfer")?
        .add_paired_category("Currency Exchange")?;

    let grocery_items = [
        "Whole Foods Market",
        "Trader Joe's",
        "Costco Wholesale",
        "Target Supercenter",
        "Local Farmers Market",
    ];
    let dining_items = [
        "Starbucks Coffee",
        "Chipotle Grill",
        "Pizzeria Luigi",
        "Sushi House",
        "Downtown Bistro",
        "Burger Joint",
    ];
    let leisure_items = [
        "Netflix Subscription",
        "Spotify Premium",
        "Steam Games",
        "Local Cinema",
        "Concert Ticket",
        "Book Store",
    ];
    let transport_items = [
        "Shell Gas Station",
        "Exxon Mobil",
        "Uber Ride",
        "City Subway Pass",
        "Train Ticket",
    ];

    let mut chained_user = user;

    // --- LOOP 1: Extended to 8 Months of Paychecks & Income (24 distinct actions) ---
    for month in 1..=8 {
        chained_user = chained_user
            .add_transaction(
                "Bi-Weekly Salary",
                Some("Tech Corp Base Pay"),
                (250000, "USD"),
                (1, month, 2026),
                "Salary & Business Income",
                "Primary Paycheck",
                "Main Checking",
            )?
            .add_transaction(
                "Bi-Weekly Salary",
                Some("Tech Corp Base Pay"),
                (250000, "USD"),
                (15, month, 2026),
                "Salary & Business Income",
                "Primary Paycheck",
                "Main Checking",
            )?
            .add_transaction(
                "Side Hustle Payout",
                Some("Contract Web Dev"),
                (65000, "USD"),
                (22, month, 2026),
                "Salary & Business Income",
                "Freelance Gig",
                "Main Checking",
            )?;
    }

    // --- LOOP 2: Extended to 8 Months of Recurring Fixed Bills (32 distinct actions) ---
    for month in 1..=8 {
        chained_user = chained_user
            .add_transaction(
                "Monthly Rent",
                Some("Apartment 4B Lease"),
                (-145000, "USD"),
                (1, month, 2026),
                "Housing & Utilities",
                "Rent & Mortgage",
                "Main Checking",
            )?
            .add_transaction(
                "Electric Grid Co.",
                Some("Utility Bill"),
                (-11520, "USD"),
                (4, month, 2026),
                "Housing & Utilities",
                "Electricity Bill",
                "Main Checking",
            )?
            .add_transaction(
                "Planet Fitness",
                Some("Monthly Membership"),
                (-2499, "USD"),
                (8, month, 2026),
                "Healthcare",
                "Gym Membership",
                "Credit Card Balance",
            )?
            .add_transaction(
                "Cross Shield",
                Some("Health Premium"),
                (-8900, "USD"),
                (10, month, 2026),
                "Healthcare",
                "Medical Insurance",
                "Main Checking",
            )?;
    }

    // --- LOOP 3: High-Frequency Variable Expenses Expanded (96 distinct actions) ---
    for month in 1..=6 {
        for week in 0..4 {
            let day_offset: u8 = week * 7;

            let grocery_name = grocery_items[week as usize % grocery_items.len()];
            let dining_name = dining_items[(week + month) as usize % dining_items.len()];
            let leisure_name = leisure_items[week as usize % leisure_items.len()];
            let transport_name = transport_items[(week * month) as usize % transport_items.len()];

            chained_user = chained_user
                .add_transaction(
                    grocery_name,
                    None,
                    (-(8500 + (week as i64 * 1250)), "USD"),
                    (3 + day_offset, month, 2026),
                    "Food & Dining",
                    "Groceries",
                    "Main Checking",
                )?
                .add_transaction(
                    transport_name,
                    Some("Weekly Commute"),
                    (-(3500 + (day_offset as i64 * 150)), "USD"),
                    (5 + day_offset, month, 2026),
                    "Transportation",
                    "Gas & Fuel",
                    "Credit Card Balance",
                )?
                .add_transaction(
                    dining_name,
                    Some("Dinner with colleagues"),
                    (-(4200 + (week as i64 * 600)), "USD"),
                    (6 + day_offset, month, 2026),
                    "Food & Dining",
                    "Restaurants",
                    "Physical Wallet",
                )?
                .add_transaction(
                    leisure_name,
                    None,
                    (-(1500 + (week as i64 * 350)), "USD"),
                    (7 + day_offset, month, 2026),
                    "Entertainment & Leisure",
                    "Movies & Streaming",
                    "Credit Card Balance",
                )?;
        }
    }

    // --- LOOP 4: High-Volume Monthly Inter-Account Shifts & Allocations (48 paired entries) ---
    for month in 1..=8 {
        chained_user = chained_user
            .add_paired_transaction(
                "Monthly Savings Sweep",
                Some("Pay Yourself First Allocation"),
                (-75000, "USD"),
                (75000, "USD"),
                (2, month, 2026),
                "Investments",
                "Internal Transfer",
                "Main Checking",
                "High-Yield Savings",
            )?
            .add_paired_transaction(
                "DCA Vanguard ETF",
                Some("SP500 Index Buy-In"),
                (-50000, "USD"),
                (50000, "USD"),
                (16, month, 2026),
                "Investments",
                "Internal Transfer",
                "Main Checking",
                "Investment Brokerage",
            )?
            .add_paired_transaction(
                "ATM Withdrawal",
                Some("Cash liquidity top-up"),
                (-20000, "USD"),
                (20000, "USD"),
                (18, month, 2026),
                "Housing & Utilities",
                "Internal Transfer",
                "Main Checking",
                "Physical Wallet",
            )?
            .add_paired_transaction(
                "Pay Credit Card Statement",
                Some("Full Statement Balance Settlement"),
                (-65000, "USD"),
                (65000, "USD"),
                (28, month, 2026),
                "Housing & Utilities",
                "Internal Transfer",
                "Main Checking",
                "Credit Card Balance",
            )?
            .add_paired_transaction(
                "Holiday FX Order",
                Some("Converting cash for travel"),
                (-450000, "MYR"),
                (100000, "JPY"),
                (20, month, 2026),
                "Entertainment & Leisure",
                "Currency Exchange",
                "Physical Wallet",
                "Physical Wallet",
            )?
            .add_paired_transaction(
                "Euro Cash Prep",
                Some("Buying European currency reserve"),
                (-22000, "USD"),
                (20000, "EUR"),
                (25, month, 2026),
                "Entertainment & Leisure",
                "Currency Exchange",
                "Main Checking",
                "Physical Wallet",
            )?;
    }

    // =========================================================================
    // 2. EXTENDED SIMULATION: QUERIES, FILTERS, SORTS & SYSTEM MUTATIONS
    // =========================================================================
    println!("\n--- STARTING ADVANCED QUERY ANALYSIS AND DATA CLEANUP ---");

    // --- STEP A: Comprehensive Chained Sort Layouts ---
    println!(
        "\n[1] Smart Budget Layout: Transactions filtered by USD, ordered by Inner Absolute Volume -> Splitting Flow +/- blocks"
    );
    chained_user
        .transactions()?
        .filter_currency("USD")
        .sort_by_abs_amount() // Step 1: Smallest volumetric change to largest
        .sort_by_flow() // Step 2: Clear split block (+ve income grouped at top, -ve below)
        .print();

    println!(
        "\n[2] Pure Volumetric Sieve: All global entries purely by scale of absolute change value"
    );
    chained_user.transactions()?.sort_by_abs_amount().print();

    // --- STEP B: Advanced Editing/Mutation Assertions ---
    println!("\n[3] Target Adjustment Scrubber: Mutating fields across single and paired items...");

    // Isolate a local target subset to fetch a reliable index row mapping
    let food_query = chained_user
        .transactions()?
        .filter_group("Food & Dining")
        .sort_by_amount();

    println!("--- Food & Dining items BEFORE field modifications ---");
    let food_query = food_query.print();

    println!(
        "Modifying entry #1 title name, changing date timeline, and altering ledger amount value..."
    );
    let food_query = food_query
        .edit_name(1, "Premium Organic Foods")?
        .edit_date(1, 28, 2, 2026)?
        .edit_amount(1, -9950)?;

    println!("--- Food & Dining items AFTER field modifications ---");
    let food_query = food_query.print();

    // --- STEP C: Relational Validation via Linked Structural Mutators ---
    println!("\n[4] Inter-Account Fund/Currency Mutation Scrubber...");
    let transit_query = chained_user
        .transactions()?
        .filter_group("Transportation")
        .sort_by_date();

    println!("--- Transportation logs before fund re-assignment ---");
    let transit_query = transit_query.print();

    println!("Reassigning underlying asset fund wallet structure for line item #2...");
    let transit_query = transit_query.edit_fund(2, "Physical Wallet")?;

    println!("Changing asset processing currency reference to Euro for line item #2...");
    let transit_query = transit_query.edit_currency(2, "EUR")?;

    println!("--- Transportation logs after fund re-assignment ---");
    transit_query.print();

    // --- STEP D: Structural Master Definitions Schema Updates ---
    println!("\n[5] Structural Metadata Modifications (Unique Tables)...");

    let groups_query = chained_user.groups()?.sort_by_name().print();
    println!("Renaming tracking group entry #4 in metadata table...");
    let groups_query = groups_query.edit_name(4, "Entertainment & Media")?;
    groups_query.print();

    // --- STEP E: Safe Category Variant Swapping Assertions ---
    println!("\n[6] Safe Structural Configuration Variants Alteration Check...");

    // Provision a brand-new, completely unlinked category to guarantee an empty state
    chained_user = chained_user.add_category("Unused Prototype Budget")?;

    // Fetch categories sorted alphabetically so we can find our targets
    let cat_query = chained_user.categories()?.sort_by_name().print();

    // 1. EXTRACT INDICES FIRST while we still have full access to cat_query.rows
    let unused_idx = cat_query
        .rows
        .iter()
        .position(|c| c.label.name == "UNUSED PROTOTYPE BUDGET")
        .map(|i| i + 1)
        .unwrap_or(1);
    let in_use_idx = cat_query
        .rows
        .iter()
        .position(|c| c.label.name == "GROCERIES")
        .map(|i| i + 1)
        .unwrap_or(2);

    println!(
        "Attempting safe change of empty category type variant state ('Unused Prototype Budget' at index {unused_idx})..."
    );
    // 2. This moves cat_query, but returns a fresh one which we bind back to `cat_query`
    let mut cat_query = cat_query.edit_variant(unused_idx, CategoryVariant::Paired)?;
    println!("Successfully swapped variant on empty category!");

    println!(
        "Attempting a blocked variant edit on a category currently mapped to active transactions ('Groceries' at index {in_use_idx})..."
    );

    // 3. Since edit_variant takes `self` by value, we must capture whatever it returns
    // inside BOTH arms of the match statement so we don't lose the value on failure.
    cat_query = match cat_query.edit_variant(in_use_idx, CategoryVariant::Paired) {
        Err(e) => {
            println!("Caught expected block validation guard successfully: {e}");
            // Return the unconsumed query collection out of the match block if it failed
            let fresh_lookup = chained_user.categories()?.sort_by_name();
            fresh_lookup
        }
        Ok(_) => {
            panic!("Validation Failure: System allowed variant change for active attached entries!")
        }
    };

    // --- STEP F: Forcing Cascaded Structural Overwrites ---
    println!("\n[7] Forced Structural Re-Balancing Overwrites...");

    // 4. Now we can cleanly pass the valid query chain into the force wrapper!
    let cat_query = cat_query.force_edit_variant(in_use_idx, CategoryVariant::Paired)?;
    cat_query.print();

    // --- STEP G: Destructive Pipeline Removals ---
    println!("\n[8] Executing Final Selective System Cleanups...");
    let ledger = chained_user.transactions()?.sort_by_date();
    println!("Purging chronological record at position #1...");
    let ledger = ledger.delete(1)?;

    println!("\n[9] Running Final Engine State System Verification Sweep...");
    ledger.print();

    println!("--- EXTENDED SIMULATION PATH EXECUTED SUCCESSFULLY ---");
    Ok(())
}
