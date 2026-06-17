mod user;
use user::*;

fn main() {
    print! {"\n\n\n\n\n\n\n\n--------------------------------------------------\n\n"};

    let user = User::new("TEST");
    user.add_group("du bist gut genug");
    user.add_group("du bist gut genug");
    user.add_currency("SGD");
    user.add_group("gift");
    user.add_category("surgery");
    user.add_fund("savings");
    User::unwrap_result(user.add_transaction(
        "bbl",
        Some("baby"),
        (1000, "SGD"),
        (1, 1, 2024),
        "gift",
        "surgery",
        "savings",
    ));

    user.add_currency("USD");
    user.add_group("Invest");
    user.add_paired_category("convert");
    user.add_fund("multicurrency");
    User::unwrap_result(user.add_paired_transaction(
        "DogeCoinGO!!!",
        Some("Neta"),
        (6767, "USD"),
        (1, 6, 2026),
        "Invest",
        "convert",
        "multicurrency",
        "multicurrency",
    ));

    print! {"\n--------------------------------------------------\n\n"};
}
