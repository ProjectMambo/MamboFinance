#![allow(dead_code)]

use crate::user::{User, UserError};

pub fn simulate_user_activity(user: &User) -> Result<(), UserError> {
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

    // Collections of items for procedural programmatic insertion
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

    // Keep track of current user context as a mutable reference to chain safely
    let mut chained_user = user;

    // --- LOOP 1: Simulate 4 Months of Paychecks & Income (12 actions) ---
    for month in 5..=8 {
        chained_user = chained_user
            .add_transaction(
                "Bi-Weekly Salary",
                Some("Tech Corp Base Pay"),
                (250000, "USD"), // Income (+ve)
                (1, month, 2026),
                "Salary & Business Income",
                "Primary Paycheck",
                "Main Checking",
            )?
            .add_transaction(
                "Bi-Weekly Salary",
                Some("Tech Corp Base Pay"),
                (250000, "USD"), // Income (+ve)
                (15, month, 2026),
                "Salary & Business Income",
                "Primary Paycheck",
                "Main Checking",
            )?
            .add_transaction(
                "Side Hustle Payout",
                Some("Contract Web Dev"),
                (65000, "USD"), // Income (+ve)
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
                (-145000, "USD"), // Expense (-ve)
                (1, month, 2026),
                "Housing & Utilities",
                "Rent & Mortgage",
                "Main Checking",
            )?
            .add_transaction(
                "Electric Grid Co.",
                Some("Utility Bill"),
                (-11520, "USD"), // Expense (-ve)
                (4, month, 2026),
                "Housing & Utilities",
                "Electricity Bill",
                "Main Checking",
            )?
            .add_transaction(
                "Planet Fitness",
                Some("Monthly Membership"),
                (-2499, "USD"), // Expense (-ve)
                (8, month, 2026),
                "Healthcare",
                "Gym Membership",
                "Credit Card Balance",
            )?
            .add_transaction(
                "Cross Shield",
                Some("Health Premium"),
                (-8900, "USD"), // Expense (-ve)
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
                // Weekly Grocery Stock (Food & Dining)
                .add_transaction(
                    grocery_name,
                    None,
                    (-(8500 + (week as i64 * 1250)), "USD"), // Expense (-ve)
                    (3 + day_offset, month, 2026),
                    "Food & Dining",
                    "Groceries",
                    "Main Checking",
                )?
                // Commute Fuel / Transit Costs (Transportation)
                .add_transaction(
                    transport_name,
                    Some("Weekly Commute"),
                    (-(3500 + (day_offset as i64 * 150)), "USD"), // Expense (-ve)
                    (5 + day_offset, month, 2026),
                    "Transportation",
                    "Gas & Fuel",
                    "Credit Card Balance",
                )?
                // Mid-week Social Dining (Leisure)
                .add_transaction(
                    dining_name,
                    Some("Dinner with colleagues"),
                    (-(4200 + (week as i64 * 600)), "USD"), // Expense (-ve)
                    (6 + day_offset, month, 2026),
                    "Food & Dining",
                    "Restaurants",
                    "Physical Wallet",
                )?
                // Digital Entertainment & Outings
                .add_transaction(
                    leisure_name,
                    None,
                    (-(1500 + (week as i64 * 350)), "USD"), // Expense (-ve)
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
            // Automated Savings Contribution
            .add_paired_transaction(
                "Monthly Savings Sweep",
                Some("Pay Yourself First Allocation"),
                (-75000, "USD"), // Source account Out (-ve)
                (75000, "USD"),  // Target account In (+ve)
                (2, month, 2026),
                "Investments",
                "Internal Transfer",
                "Main Checking",
                "High-Yield Savings",
            )?
            // Long Term Market Investment DCA
            .add_paired_transaction(
                "DCA Vanguard ETF",
                Some("SP500 Index Buy-In"),
                (-50000, "USD"), // Source account Out (-ve)
                (50000, "USD"),  // Target account In (+ve)
                (16, month, 2026),
                "Investments",
                "Internal Transfer",
                "Main Checking",
                "Investment Brokerage",
            )?
            // Cash Withdrawal for Pocket money
            .add_paired_transaction(
                "ATM Withdrawal",
                Some("Cash liquidity top-up"),
                (-20000, "USD"), // Source account Out (-ve)
                (20000, "USD"),  // Target account In (+ve)
                (18, month, 2026),
                "Housing & Utilities",
                "Internal Transfer",
                "Main Checking",
                "Physical Wallet",
            )?
            // Credit Card Statement Clearing
            .add_paired_transaction(
                "Pay Credit Card Statement",
                Some("Full Statement Balance Settlement"),
                (-65000, "USD"), // Source account Out (-ve)
                (65000, "USD"),  // Target account In (+ve)
                (28, month, 2026),
                "Housing & Utilities",
                "Internal Transfer",
                "Main Checking",
                "Credit Card Balance",
            )?
            // International Travel Cash Exchanges
            .add_paired_transaction(
                "Holiday FX Order",
                Some("Converting cash for travel"),
                (-450000, "MYR"), // Source account Out (-ve)
                (100000, "JPY"),  // Target account In (+ve)
                (20, month, 2026),
                "Entertainment & Leisure",
                "Currency Exchange",
                "Physical Wallet",
                "Physical Wallet",
            )?
            .add_paired_transaction(
                "Euro Cash Prep",
                Some("Buying European currency reserve"),
                (-22000, "USD"), // Source account Out (-ve)
                (20000, "EUR"),  // Target account In (+ve)
                (25, month, 2026),
                "Entertainment & Leisure",
                "Currency Exchange",
                "Main Checking",
                "Physical Wallet",
            )?;
    }

    chained_user.groups()?.sort_by_name().print();
    chained_user
        .categories()?
        .sort_by_name()
        .sort_by_type()
        .print();
    chained_user.funds()?.sort_by_name().print();
    chained_user.currencies()?.sort_by_name().print();
    chained_user
        .transactions()?
        .sort_by_name()
        .sort_by_amount()
        .sort_by_currency()
        .print();

    Ok(())
}
