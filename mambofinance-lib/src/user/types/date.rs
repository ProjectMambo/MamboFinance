use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use thiserror::Error;

// Static lookups for calendar month validation and formatting
const MONTHS_DAY_COUNT: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTHS_NAME: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

/// Represents a validated calendar date optimized for local accounting logs.
#[derive(Copy, Clone)]
pub struct Date {
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

impl Date {
    /// Constructs and validates a new `Date` instance.
    ///
    /// # Errors
    ///
    /// Returns a `DateError` if the month is out of range or the day exceeds the days in that month.
    pub fn new(day: u8, month: u8, year: u16) -> Result<Self, DateError> {
        if month > 12 {
            return Err(DateError::InvalidMonth(month));
        }

        // Account for leap years during February validation
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

    /// Maps a single SQLite row to a `Date` instance using explicit database sequence indices.
    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Date {
            day: row.get::<_, i64>(offset)? as u8,
            month: row.get::<_, i64>(offset + 1)? as u8,
            year: row.get::<_, i64>(offset + 2)? as u16,
        })
    }

    /// Evaluates if a given year is a Gregorian leap year.
    pub fn is_leap_year(year: u16) -> bool {
        (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
    }
}

impl Display for Date {
    /// Formats the date to a fixed structural layout.
    ///
    /// e.g., "01-JAN-2026".
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:0>2}-{}-{:0>4}",
            self.day,
            MONTHS_NAME[(self.month - 1) as usize],
            self.year
        )
    }
}

impl Eq for Date {}

impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year && self.month == other.month && self.day == other.day
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.year, self.month, self.day).cmp(&(other.year, other.month, other.day))
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Compilation of calendar domain validation errors.
#[derive(Error, Debug)]
pub enum DateError {
    #[error("{0} is not a valid month, there's only 12.")]
    InvalidMonth(u8),

    #[error("There's only {0} days in {1}")]
    InvalidDay(u8, String),
}
