use std::fmt::{Display, Formatter};
use thiserror::Error;

const MONTHS_DAY_COUNT: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTHS_NAME: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

#[derive(Clone, Debug)]
pub struct Date {
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

#[derive(Error, Debug)]
pub enum DateError {
    #[error("{0} is not a valid month, there's only 12.")]
    InvalidMonth(u8),

    #[error("There's only {0} days in {1}")]
    InvalidDay(u8, String),
}

impl Date {
    pub fn new(day: u8, month: u8, year: u16) -> Result<Self, DateError> {
        if month > 12 {
            return Err(DateError::InvalidMonth(month));
        }

        let max: u8 = if month == 2 && Self::is_leap_year(year) {
            29
        } else {
            MONTHS_DAY_COUNT[(month - 1) as usize]
        };

        if day > max {
            return Err(DateError::InvalidDay(
                day,
                String::from(MONTHS_NAME[(month - 1) as usize]),
            ));
        }

        Ok(Self { day, month, year })
    }

    pub fn is_leap_year(year: u16) -> bool {
        (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if f.alternate() {
            return write!(f, "{}-{}-{}", self.day, self.month, self.year);
        }

        write!(
            f,
            "{}-{}-{}",
            self.day,
            MONTHS_NAME[(self.month - 1) as usize],
            self.year
        )
    }
}
