use std::cell::Cell;

use color_eyre::Result;
use mambofinance_lib::user::{User, UserError};
use ratatui::DefaultTerminal;

use crate::widgets::{TabState, UIState, user_list::UserListState};

#[derive(Debug)]
pub struct App {
    pub user: User,
    pub ui_state: UIState,
    pub input_override: Cell<bool>,
    pub should_quit: bool,
}

impl App {
    pub fn new(name: &str) -> Result<Self, UserError> {
        let user = User::new_in_memory(name)?;

        let user_list_state = UserListState::new(&user)?;
        let tabs: Vec<TabState> = vec![TabState::UserList(user_list_state)];

        let ui_state = UIState::new(tabs);

        Ok(App {
            user,
            ui_state,
            input_override: Cell::new(false),
            should_quit: false,
        })
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        test_user_flow(&self.user)?;
        for _ in 0..20 {
            test_add_transaction(&self.user)?;
        }

        if let Some(tab) = self.ui_state.get_mut() {
            tab.update_data(&self.user)?;
        }

        while !self.should_quit {
            terminal.draw(|f| crate::ui::render(f, &mut self))?;
            crate::update::handle_events(&mut self)?;
        }
        Ok(())
    }
}

pub struct AppContext<'a> {
    pub user: &'a mut User,
    pub input_override: &'a Cell<bool>,
}

impl<'a> AppContext<'a> {
    pub fn is_override(&self) -> bool {
        self.input_override.get()
    }

    pub fn input(&self, is_override: bool) {
        self.input_override.set(is_override);
    }
}

fn test_user_flow(user: &User) -> Result<(), UserError> {
    // Currencies
    user.add_currency("USD")?
        .add_currency("EUR")?
        .add_currency("GBP")?
        .add_currency("JPY")?
        .add_currency("CAD")?
        .add_currency("AUD")?
        .add_currency("CHF")?
        .add_currency("CNY")?
        .add_currency("SGD")?
        .add_currency("HKD")?;

    // Funds
    user.add_fund("Physical Wallet")?
        .add_fund("Checking Account")?
        .add_fund("Savings Account")?
        .add_fund("Credit Card Platinum")?
        .add_fund("Investment Brokerage")?
        .add_fund("Crypto Cold Wallet")?
        .add_fund("PayPal Balance")?
        .add_fund("Cash Vault")?
        .add_fund("Corporate Expense Account")?
        .add_fund("Emergency Rainy Day Fund")?;

    // Groups
    user.add_group("Personal Daily")?
        .add_group("Business Operations")?
        .add_group("Summer Vacation 2026")?
        .add_group("Home Renovation")?
        .add_group("Investment Portfolio")?
        .add_group("Hobbies & Leisure")?
        .add_group("Monthly Fixed Bills")?
        .add_group("Health & Medical")?
        .add_group("Education & Books")?
        .add_group("Charity & Gifts")?;

    // Categories
    user.add_category("Groceries")?
        .add_category("Coffee & Snacks")?
        .add_category("Salary Income")?
        .add_category("Freelance Consulting")?
        .add_category("Streaming Subscriptions")?
        .add_category("Electricity Bill")?
        .add_category("Gym Membership")?
        .add_category("Dining Out")?
        .add_category("Gasoline & Fuel")?
        .add_category("Public Transit")?
        .add_category("Concert Tickets")?
        .add_category("Gadgets & Tech")?
        .add_category("Rent Payment")?
        .add_category("Stock Dividends")?
        .add_category("Dental Checkup")?
        .add_category("Books & Courses")?
        .add_category("Pet Supplies")?
        .add_category("Clothing")?
        .add_category("Home Tools")?
        .add_category("Donations")?;

    // Paired Categories
    user.add_paired_category("Internal Bank Transfer")?
        .add_paired_category("Currency Exchange Match")?
        .add_paired_category("Credit Card Payoff")?
        .add_paired_category("Brokerage Deposit")?
        .add_paired_category("Crypto Fiat Gateway")?;

    Ok(())
}

fn test_add_transaction(user: &User) -> Result<(), UserError> {
    // Single-Entry Transactions
    user.add_transaction(
        "Bi-Weekly Paycheck",
        Some("Primary corporate salary deposit"),
        (3500, "USD"),
        (1, 6, 2026),
        "Monthly Fixed Bills",
        "Salary Income",
        "Checking Account",
    )?;

    user.add_transaction(
        "Whole Foods Market",
        Some("Weekly grocery run including meal prep"),
        (-142, "USD"),
        (2, 6, 2026),
        "Personal Daily",
        "Groceries",
        "Checking Account",
    )?;

    user.add_transaction(
        "Blue Bottle Coffee",
        None,
        (-6, "USD"),
        (2, 6, 2026),
        "Personal Daily",
        "Coffee & Snacks",
        "Physical Wallet",
    )?;

    user.add_transaction(
        "Netflix Premium",
        Some("Automatic monthly renewal"),
        (-23, "USD"),
        (3, 6, 2026),
        "Monthly Fixed Bills",
        "Streaming Subscriptions",
        "Credit Card Platinum",
    )?;

    user.add_transaction(
        "Spotify Family",
        None,
        (-17, "USD"),
        (3, 6, 2026),
        "Monthly Fixed Bills",
        "Streaming Subscriptions",
        "Credit Card Platinum",
    )?;

    user.add_transaction(
        "Shell Gas Station",
        Some("Full tank premium unleaded"),
        (-55, "USD"),
        (4, 6, 2026),
        "Personal Daily",
        "Gasoline & Fuel",
        "Credit Card Platinum",
    )?;

    user.add_transaction(
        "Consulting Retainer",
        Some("External UI architecture contract review"),
        (1200, "EUR"),
        (5, 6, 2026),
        "Business Operations",
        "Freelance Consulting",
        "PayPal Balance",
    )?;

    user.add_transaction(
        "City Power & Light",
        Some("May electricity usage metrics"),
        (-115, "USD"),
        (6, 6, 2026),
        "Monthly Fixed Bills",
        "Electricity Bill",
        "Checking Account",
    )?;

    user.add_transaction(
        "Trattoria Dinner",
        Some("Team dinner with clients"),
        (-210, "EUR"),
        (7, 6, 2026),
        "Business Operations",
        "Dining Out",
        "Corporate Expense Account",
    )?;

    user.add_transaction(
        "Iron Gym Inc",
        Some("Monthly access pass"),
        (-80, "USD"),
        (8, 6, 2026),
        "Monthly Fixed Bills",
        "Gym Membership",
        "Credit Card Platinum",
    )?;

    user.add_transaction(
        "Subway Transit Metro",
        None,
        (-3, "USD"),
        (9, 6, 2026),
        "Personal Daily",
        "Public Transit",
        "Physical Wallet",
    )?;

    user.add_transaction(
        "Rock Festival Pass",
        Some("Early bird weekend wristbands"),
        (-250, "GBP"),
        (10, 6, 2026),
        "Summer Vacation 2026",
        "Concert Tickets",
        "Credit Card Platinum",
    )?;

    user.add_transaction(
        "Apple Store",
        Some("Replacement USB-C charging bricks"),
        (-39, "USD"),
        (11, 6, 2026),
        "Personal Daily",
        "Gadgets & Tech",
        "Credit Card Platinum",
    )?;

    user.add_transaction(
        "Downtown Loft Rent",
        Some("Automated ACH lease payment"),
        (-2100, "USD"),
        (12, 6, 2026),
        "Monthly Fixed Bills",
        "Rent Payment",
        "Checking Account",
    )?;

    user.add_transaction(
        "Global Tech ETF",
        Some("Quarterly dividend distribution pay out"),
        (340, "USD"),
        (13, 6, 2026),
        "Investment Portfolio",
        "Stock Dividends",
        "Investment Brokerage",
    )?;

    user.add_transaction(
        "Apex Dental Care",
        Some("Routine clean and x-ray mapping"),
        (-150, "USD"),
        (14, 6, 2026),
        "Health & Medical",
        "Dental Checkup",
        "Checking Account",
    )?;

    user.add_transaction(
        "O'Reilly Media Systems",
        Some("Rust concurrency systems design guidebook"),
        (-65, "USD"),
        (15, 6, 2026),
        "Education & Books",
        "Books & Courses",
        "Checking Account",
    )?;

    user.add_transaction(
        "Pet Health Depot",
        Some("Premium kibble organic formulation"),
        (-88, "USD"),
        (16, 6, 2026),
        "Personal Daily",
        "Pet Supplies",
        "Checking Account",
    )?;

    user.add_transaction(
        "Boutique Clothiers",
        Some("Summer linen shirts collection"),
        (-180, "EUR"),
        (17, 6, 2026),
        "Summer Vacation 2026",
        "Clothing",
        "Credit Card Platinum",
    )?;

    user.add_transaction(
        "Home Depot Supply",
        Some("Drywall anchors and heavy duty screws"),
        (-22, "USD"),
        (18, 6, 2026),
        "Home Renovation",
        "Home Tools",
        "Checking Account",
    )?;

    user.add_transaction(
        "Red Cross Global",
        None,
        (-100, "USD"),
        (19, 6, 2026),
        "Charity & Gifts",
        "Donations",
        "Checking Account",
    )?;

    user.add_transaction(
        "Corner Bakery Bistro",
        Some("Quick morning sourdough loaves"),
        (-12, "USD"),
        (20, 6, 2026),
        "Personal Daily",
        "Coffee & Snacks",
        "Physical Wallet",
    )?;

    // Paired Transactions
    user.add_paired_transaction(
        "Liquidity Optimization",
        Some("Moving operational cash to savings allocation"),
        (-1000, "USD"),
        (1000, "USD"),
        (21, 6, 2026),
        "Monthly Fixed Bills",
        "Internal Bank Transfer",
        "Checking Account",
        "Savings Account",
    )?;

    user.add_paired_transaction(
        "Travel FX Buffer Setup",
        Some("Converting baseline USD assets to EUR travel cash"),
        (-500, "USD"),
        (450, "EUR"),
        (21, 6, 2026),
        "Summer Vacation 2026",
        "Currency Exchange Match",
        "Checking Account",
        "Physical Wallet",
    )?;

    user.add_paired_transaction(
        "Statement Clear",
        Some("Clearing down running liabilities balances"),
        (-650, "USD"),
        (650, "USD"),
        (22, 6, 2026),
        "Monthly Fixed Bills",
        "Credit Card Payoff",
        "Checking Account",
        "Credit Card Platinum",
    )?;

    user.add_paired_transaction(
        "Capital Deployment Shift",
        Some("Injecting funds into index tracking pools"),
        (-1500, "USD"),
        (1500, "USD"),
        (22, 6, 2026),
        "Investment Portfolio",
        "Brokerage Deposit",
        "Checking Account",
        "Investment Brokerage",
    )?;

    user.add_paired_transaction(
        "Crypto Exposure Accumulation",
        Some("Converting checking liquidity into digital assets directly"),
        (-800, "USD"),
        (800, "USD"),
        (23, 6, 2026),
        "Investment Portfolio",
        "Crypto Fiat Gateway",
        "Checking Account",
        "Crypto Cold Wallet",
    )?;

    user.add_paired_transaction(
        "ATM Settlement Extraction",
        Some("Withdrawing hard currency for safety storage reserves"),
        (-300, "USD"),
        (300, "USD"),
        (23, 6, 2026),
        "Personal Daily",
        "Internal Bank Transfer",
        "Checking Account",
        "Cash Vault",
    )?;

    Ok(())
}
