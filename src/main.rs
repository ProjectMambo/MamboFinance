mod user;
use user::*;

fn main() -> Result<(), UserError> {
    print! {"\n\n\n\n\n\n\n\n--------------------------------------------------\n\n"};

    let user = User::new("TEST")?;
    user.add_group("du bist gut genug")?
        .add_currency("SGD")?
        .add_group("gift")?
        .add_category("surgery")?
        .add_fund("savings")?
        .add_transaction(
            "bbl",
            Some("baby"),
            (-1000, "SGD"),
            (1, 1, 2024),
            "gift",
            "surgery",
            "savings",
        )?
        .add_currency("USD")?
        .add_group("Invest")?
        .add_paired_category("convert")?
        .add_fund("multicurrency")?
        .add_paired_transaction(
            "DogeCoinGO!!!",
            Some("Neta"),
            (-6767, "SGD"),
            (1738, "USD"),
            (1, 6, 2026),
            "Invest",
            "convert",
            "multicurrency",
            "multicurrency",
        )?
        .print_transaction()?
        .print_group()?
        .print_category()?
        .print_fund()?
        .print_currency()?;

    print! {"\n--------------------------------------------------\n\n"};
    Ok(())
}
