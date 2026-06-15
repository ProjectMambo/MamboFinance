use crate::define_struct;
use std::fmt::{Display, Formatter};

define_struct!(
Date with {
    day: u8,
    month: u8,
    year: u16,
} check {
    if month > 12 {
        return Err(String::from("There's only 12 months!"));
    }

    let mut months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    if month == 2 && ((year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)){
        months[1] = 29;
    }

    let max: u8 = months[(month - 1) as usize];

    if day > max {
        return Err(format!("There's only {} days in this month!", max))
    }
});

impl Display for Date {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let months: [&str; 12] = [
            "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
        ];

        write!(
            f,
            "{}-{}-{}",
            self.day,
            months[(self.month - 1) as usize],
            self.year
        )
    }
}
