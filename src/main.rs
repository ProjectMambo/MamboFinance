// enum Flow {
//     In,
//     Out,
// }

// struct USD;
// struct SGD;

mod core;
mod user;
use core::*;
use user::*;

use crate::user::CategoryVariant::*;

fn main() {
    let test_currency: Currency = Currency::new(String::from("SGD")).unwrap();
    let test_amount: Amount = Amount::new(test_currency, -67).unwrap();
    let test_date: Date = Date::new(2, 5, 2026).unwrap();

    let test_group: Group =
        Group::new(Label::new(21, String::from("testGroup"), None).unwrap()).unwrap();
    let test_category: Category = Category::new(
        Label::new(333, String::from("du bist gut genug"), None).unwrap(),
        Paired,
    )
    .unwrap();
    let test_fund: Fund =
        Fund::new(Label::new(4444, String::from("du bist gut genug"), None).unwrap()).unwrap();

    let t = Transaction::new(
        Label::new(67i32, String::from("Test"), Some(String::from("testdes"))).unwrap(),
        test_amount,
        test_date,
        test_group,
        test_category,
        test_fund,
        None,
    )
    .unwrap();

    println!("{}", t)
}
