#![allow(dead_code)]

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

    // --- LOOP 1: Simulate 4 Months of Paychecks & Income (12 actions) ---
    for month in 5..=8 {
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

    // --- LOOP 2: Simulate Recurring Monthly Fixed Bills (16 actions) ---
    for month in 5..=8 {
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

    // --- LOOP 3: High-Frequency Variable Expenses (60 actions) ---
    for month in 5..=7 {
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

    // --- LOOP 4: Monthly Financial Adjustments & Inter-Account Shifts (24 actions) ---
    for month in 5..=8 {
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
    // 2. EXTENDED SIMULATION: QUERIES, FILTERS, SORTS & SYSTEM DELETIONS
    // =========================================================================
    println!("\n--- STARTING ADVANCED QUERY ANALYSIS AND DATA CLEANUP ---");

    // --- STEP A: Initial Structural Grouping Outputs ---
    println!("\n[1] Rendering structural settings catalogs sorted...");
    chained_user.groups()?.sort_by_name().print();
    chained_user
        .categories()?
        .sort_by_name()
        .sort_by_type()
        .print();
    chained_user.funds()?.sort_by_name().print();
    chained_user.currencies()?.sort_by_name().print();

    // --- STEP B: High Volume Multi-Stage Sorting & Isolation Checks ---
    println!("\n[2] Master Ledger: All transactions ordered by Currency -> Amount -> Name");
    chained_user
        .transactions()?
        .sort_by_name()
        .sort_by_amount()
        .sort_by_currency()
        .print();

    println!("\n[3] Historical Timeline: All transactions ordered by Date chronologically");
    chained_user.transactions()?.sort_by_date().print();

    // --- STEP C: Deep Multi-layered Filtering Pipelines ---
    println!("\n[4] Isolated Deep Dive: Food & Dining outlays using 'Main Checking'...");
    chained_user
        .transactions()?
        .filter_group("Food & Dining")
        .filter_fund("Main Checking")
        .sort_by_amount()
        .print();

    println!("\n[5] Investment Activity Radar: Brokerage allocations across any currency asset...");
    chained_user
        .transactions()?
        .filter_group("Investments")
        .filter_fund("Investment Brokerage")
        .sort_by_date()
        .print();

    println!("\n[6] Foreign Currency Operations: Isolating multi-currency JPY positions...");
    chained_user
        .transactions()?
        .filter_currency("JPY")
        .sort_by_amount()
        .print();

    // --- STEP D: Executing System Data Cleanups via Query Index Deletion ---
    println!(
        "\n[7] Target Scrubber: Deleting a specific Transaction entry and tracking linkage changes..."
    );
    // Grab the current sorted listing window scope to capture index context safely
    let tx_query = chained_user
        .transactions()?
        .filter_group("Housing & Utilities")
        .sort_by_amount();
    println!("--- Housing & Utilities items BEFORE indexed execution removal ---");
    let tx_query = tx_query.print();

    // Perform index item extraction (Targeting index row 2 from current selection window)
    println!("Executing cascading linked transaction erasure at index position 2...");
    let tx_query = tx_query.delete(2)?;
    println!("--- Housing & Utilities items AFTER index execution removal ---");
    tx_query.print();

    // --- STEP E: Struct Balancing Cleanup Operations ---
    println!("\n[8] Configuration Scrubber: Removing unused structural criteria items...");

    // View current funds setup
    let fund_query = chained_user.funds()?.sort_by_name().print();
    println!("Removing index item #3 from active account funds options...");
    fund_query.delete(3)?.print();

    // View current categories setup
    let cat_query = chained_user.categories()?.sort_by_name().print();
    println!("Removing item entry #5 from system category variants definitions...");
    cat_query.delete(5)?.print();

    // View current groupings layout
    let group_query = chained_user.groups()?.sort_by_name().print();
    println!("Removing entry configuration mapping item #4 from active tracking layout groups...");
    group_query.delete(4)?.print();

    // --- STEP F: Final Integrity Reporting Check ---
    println!("\n[9] Running Final Engine State System Verification Sweep...");
    chained_user.transactions()?.sort_by_date().print();

    println!("--- SIMULATION PATH EXECUTED SUCCESSFULLY ---");
    Ok(())
}
