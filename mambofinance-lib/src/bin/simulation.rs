#![allow(dead_code, unused)]

use mambofinance_lib::user::{CategoryVariant, User, UserError};

fn main() -> Result<(), UserError> {
    let user = &User::new_in_memory("Test")?;

    // =========================================================================
    // SECTION 1: MASTER DATA SEEDING
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
        .add_group("Education & Self-Improvement")?
        .add_group("Miscellanous Mishaps")?
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
        .add_category("Books & Courses")?
        .add_category("Fines & Fees")?
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

    // --- LOOP 1: Regular Paychecks & Secondary Revenue Streams ---
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

    // --- LOOP 2: Fixed Monthly Overhead & Operational Bills ---
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

    // --- LOOP 3: High-Frequency Variable Expenses ---
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

    // --- LOOP 4: Paired Inter-Account Balancing & Currency Swaps ---
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
    // SECTION 2: READ-ONLY REPORT VIEWS & ADVANCED PIPELINES
    // =========================================================================
    println!("\n--- STARTING ADVANCED QUERY ANALYSIS AND DATA CLEANUP ---");

    println!(
        "\n[1] Smart Budget Layout: Filtered USD, ordered by Abs Value -> Split Flow +/- Blocks"
    );
    chained_user
        .transactions()?
        .filter_currency("USD")
        .sort_by_abs_amount()
        .sort_by_flow()
        .print();

    println!(
        "\n[2] Pure Volumetric Sieve: Global entries sorted by scale of absolute variance (Descending via sort_reverse)"
    );
    chained_user
        .transactions()?
        .sort_by_abs_amount()
        .sort_reverse()
        .print();

    // =========================================================================
    // SECTION 3: TRANSACTION FIELD MUTATIONS
    // =========================================================================
    println!(
        "\n[3] Target Scrubber: Running sequential field adjustments via dynamic re-queries..."
    );

    let mut food_query = chained_user
        .transactions()?
        .filter_group("Food & Dining")
        .sort_by_amount();
    println!("--- Food & Dining items BEFORE field modifications ---");
    food_query = food_query.print();

    // Apply edits sequentially, capturing the returned Query wrappers to keep memory layouts synced
    food_query = food_query.edit_name(1, "Premium Organic Foods")?;
    food_query = food_query.edit_date(1, 28, 2, 2026)?;
    food_query = food_query.edit_amount(1, -9950)?;

    // Perform a complete structural database re-query verification check
    let food_query_final = chained_user
        .transactions()?
        .filter_group("Food & Dining")
        .sort_by_amount();
    println!("--- Food & Dining items AFTER field modifications (RE-QUERIED FROM DB) ---");
    food_query_final.print();

    // =========================================================================
    // SECTION 4: ASSET WALLET & FILTER REVERSAL DEMO
    // =========================================================================
    println!("\n[4] Inter-Account Fund & Currency Asset Refactoring + Filter Reversal...");

    let mut transit_query = chained_user
        .transactions()?
        .filter_group("Transportation")
        .sort_by_date();
    println!("--- Transportation logs before fund re-assignment ---");
    transit_query = transit_query.print();

    // Apply Fund & Currency adjustments across the running query lifecycle safely
    transit_query = transit_query.edit_fund(2, "Physical Wallet")?;
    transit_query = transit_query.edit_currency(2, "EUR")?;

    // Hard drop layout context and verify database reality
    let transit_query_final = chained_user
        .transactions()?
        .filter_group("Transportation")
        .sort_by_date();
    println!("--- Transportation logs after fund re-assignment (RE-QUERIED FROM DB) ---");
    transit_query_final.print();

    println!("\n[4b] Testing Filter Inversion Engine:");
    println!("Printing everything EXCEPT Transportation (Using filter_reverse)...");
    chained_user
        .transactions()?
        .filter_group("Transportation") // Isolates Transportation (shelves everything else)
        .filter_reverse() // Swaps visible records with background shelved storage
        .sort_by_date() // Sorts the newly active dataset
        .print();

    // =========================================================================
    // SECTION 5: METADATA SYSTEM DEFINITIONS
    // =========================================================================
    println!("\n[5] Structural Master Metadata Modifications...");

    let mut groups_query = chained_user.groups()?.sort_by_name();
    println!("--- Groups Table BEFORE modification ---");
    groups_query = groups_query.print();

    groups_query = groups_query.edit_name(4, "Entertainment & Media")?;

    // Pull a fresh reflection from unique metadata schemas
    let groups_query_final = chained_user.groups()?.sort_by_name();
    println!("--- Groups Table AFTER modification (RE-QUERIED FROM DB) ---");
    groups_query_final.print();

    // =========================================================================
    // SECTION 6: CONFIGURATION VARIANT SAFETY SWAPS
    // =========================================================================
    println!("\n[6] Safe Structural Configuration Variants Alteration Check...");

    chained_user = chained_user.add_category("Unused Prototype Budget")?;

    let mut cat_query = chained_user.categories()?.sort_by_name();
    println!("--- Current Structural Categories ---");
    cat_query = cat_query.print();

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
        "Attempting safe change of empty category variant state ('Unused Prototype Budget' at index {unused_idx})..."
    );
    cat_query = cat_query.edit_variant(unused_idx, CategoryVariant::Paired)?;
    println!("Successfully swapped variant on empty category!");

    println!(
        "Attempting blocked variant change on category bound to active records ('Groceries' at index {in_use_idx})..."
    );
    cat_query = match cat_query.edit_variant(in_use_idx, CategoryVariant::Paired) {
        Err(e) => {
            println!("Caught expected block validation guard successfully: {e}");
            // Re-fetch category configuration out of DB to refresh layout loop safely
            chained_user.categories()?.sort_by_name()
        }
        Ok(_) => {
            panic!("Validation Failure: System allowed variant change for active attached entries!")
        }
    };

    // =========================================================================
    // SECTION 7: CASCADING FORCE OVERWRITES
    // =========================================================================
    println!("\n[7] Forced Structural Re-Balancing Overwrites...");

    // Force edit structural layout mutations and catch ownership loop context safely
    cat_query = cat_query.force_edit_variant(in_use_idx, CategoryVariant::Paired)?;

    // Pull pristine snapshot verification from Single Source of Truth
    let cat_query_post_force = chained_user.categories()?.sort_by_name();
    println!("--- Categories after Forced Cascade Overwrite (RE-QUERIED FROM DB) ---");
    cat_query_post_force.print();

    // =========================================================================
    // SECTION 8: RECORD PURGING & DELETION PIPELINES
    // =========================================================================
    println!("\n[8] Executing Final Selective System Cleanups...");

    let ledger = chained_user.transactions()?.sort_by_date();
    println!("Purging chronological record row at position #1...");

    // Execute destructive action (ignoring returned layout representation as we pull raw DB state next)
    let _ = ledger.delete(1)?;

    // =========================================================================
    // SECTION 9: FINAL VERIFICATION AUDIT
    // =========================================================================
    println!("\n[9] Running Final Engine State System Verification Sweep...");

    let final_ledger_snapshot = chained_user.transactions()?.sort_by_date();
    final_ledger_snapshot.print();

    println!("--- EXTENDED SIMULATION PATH EXECUTED SUCCESSFULLY ---");
    Ok(())
}
