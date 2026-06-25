use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use thiserror::Error;

// Static lookups for calendar month validation and formatting
const MONTHS_DAY_COUNT: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTHS_NAME: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

/// Represents a validated calendar date optimized for local accounting logs.
#[derive(Copy, Clone, Debug)]
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

// region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // region: Date::new

    /// Verifies that `Date::new` successfully constructs a struct from valid parameters.
    #[test]
    fn new_accepts_a_valid_date() {
        // Arrange
        // Act
        let result = Date::new(15, 6, 2026);

        // Assert
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!((date.day, date.month, date.year), (15, 6, 2026));
    }

    /// Verifies that `Date::new` rejects an invalid month index that exceeds 12.
    #[test]
    fn new_rejects_month_above_twelve() {
        // Arrange
        // Act
        let result = Date::new(1, 13, 2026);

        // Assert
        assert!(matches!(result, Err(DateError::InvalidMonth(13))));
    }

    /// Verifies that December is accepted as a valid upper boundary for months.
    #[test]
    fn new_accepts_month_exactly_twelve() {
        // Arrange
        // Act
        let result = Date::new(31, 12, 2026);

        // Assert
        assert!(result.is_ok());
    }

    /// Verifies that a day parameter exceeding the month's maximum length throws a validation error.
    #[test]
    fn new_rejects_day_exceeding_days_in_month() {
        // Arrange
        // Act
        let result = Date::new(31, 4, 2026); // April has 30 days

        // Assert
        assert!(matches!(result, Err(DateError::InvalidDay(31, _))));
    }

    /// Verifies that February 29 is allowed when the year matches leap parameters.
    #[test]
    fn new_accepts_february_29_on_leap_year() {
        // Arrange
        // Act
        let result = Date::new(29, 2, 2024);

        // Assert
        assert!(result.is_ok());
    }

    /// Verifies that February 29 is blocked when the year does not match leap parameters.
    #[test]
    fn new_rejects_february_29_on_non_leap_year() {
        // Arrange
        // Act
        let result = Date::new(29, 2, 2025);

        // Assert
        assert!(matches!(result, Err(DateError::InvalidDay(29, _))));
    }

    /// Verifies that February 28 is cleanly processed inside standard non-leap years.
    #[test]
    fn new_accepts_february_28_on_non_leap_year() {
        // Arrange
        // Act
        let result = Date::new(28, 2, 2025);

        // Assert
        assert!(result.is_ok());
    }

    // endregion

    // region: Date::from_row_offset

    /// Verifies that row parsing can pull sequential day, month, and year values out of standard database structures.
    #[test]
    fn from_row_offset_maps_day_month_year_columns() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE dates (day INTEGER, month INTEGER, year INTEGER);",
            (),
        )
        .expect("failed to create table");
        conn.execute(
            "INSERT INTO dates (day, month, year) VALUES (5, 9, 2026)",
            (),
        )
        .expect("failed to insert row");

        // Act
        let date: Date = conn
            .query_row("SELECT day, month, year FROM dates", (), |row| {
                Date::from_row_offset(row, 0)
            })
            .expect("query should succeed");

        // Assert
        assert_eq!((date.day, date.month, date.year), (5, 9, 2026));
    }

    /// Verifies that database deserialization functions process fields at advanced index configurations.
    #[test]
    fn from_row_offset_respects_a_nonzero_column_offset() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE dates (padding INTEGER, day INTEGER, month INTEGER, year INTEGER);",
            (),
        )
        .expect("failed to create table");
        conn.execute(
            "INSERT INTO dates (padding, day, month, year) VALUES (1, 5, 9, 2026)",
            (),
        )
        .expect("failed to insert row");

        // Act
        let date: Date = conn
            .query_row("SELECT padding, day, month, year FROM dates", (), |row| {
                Date::from_row_offset(row, 1)
            })
            .expect("query should succeed");

        // Assert
        assert_eq!((date.day, date.month, date.year), (5, 9, 2026));
    }

    // endregion

    // region: Date::is_leap_year

    /// Verifies leap determination returns true for basic cycles divisible by 4.
    #[test]
    fn is_leap_year_true_for_year_divisible_by_four() {
        // Arrange
        // Act
        let result = Date::is_leap_year(2024);

        // Assert
        assert!(result);
    }

    /// Verifies leap determination returns false for years outside standard 4-year splits.
    #[test]
    fn is_leap_year_false_for_year_not_divisible_by_four() {
        // Arrange
        // Act
        let result = Date::is_leap_year(2025);

        // Assert
        assert!(!result);
    }

    /// Verifies that exceptional century rules return false for baseline 100-year thresholds.
    #[test]
    fn is_leap_year_false_for_century_year_not_divisible_by_400() {
        // Arrange
        // Act
        let result = Date::is_leap_year(1900);

        // Assert
        assert!(!result);
    }

    /// Verifies that exceptional century rules return true for overriding 400-year checkpoints.
    #[test]
    fn is_leap_year_true_for_century_year_divisible_by_400() {
        // Arrange
        // Act
        let result = Date::is_leap_year(2000);

        // Assert
        assert!(result);
    }

    // endregion

    // region: PartialEq / Eq for Date

    /// Verifies matching instances evaluate as equivalent under binary structural operations.
    #[test]
    fn equality_holds_for_identical_dates() {
        // Arrange
        let a = Date::new(1, 1, 2026).unwrap();
        let b = Date::new(1, 1, 2026).unwrap();

        // Act
        let is_equal = a == b;

        // Assert
        assert!(is_equal);
    }

    /// Verifies differing structures fail direct comparative equal queries.
    #[test]
    fn equality_fails_for_different_days() {
        // Arrange
        let a = Date::new(1, 1, 2026).unwrap();
        let b = Date::new(2, 1, 2026).unwrap();

        // Act
        let is_equal = a == b;

        // Assert
        assert!(!is_equal);
    }

    // endregion

    // region: Ord / PartialOrd for Date

    /// Verifies sorting prioritization evaluates year bounds before lower components.
    #[test]
    fn ordering_compares_by_year_first() {
        // Arrange
        let earlier = Date::new(31, 12, 2025).unwrap();
        let later = Date::new(1, 1, 2026).unwrap();

        // Act
        let is_earlier_less = earlier < later;

        // Assert
        assert!(is_earlier_less);
    }

    /// Verifies sorting fallback tracks internal months when wrapping years are completely identical.
    #[test]
    fn ordering_compares_by_month_when_year_matches() {
        // Arrange
        let earlier = Date::new(28, 1, 2026).unwrap();
        let later = Date::new(1, 2, 2026).unwrap();

        // Act
        let is_earlier_less = earlier < later;

        // Assert
        assert!(is_earlier_less);
    }

    /// Verifies sorting resolution matches days when outer year and month containers lock.
    #[test]
    fn ordering_compares_by_day_when_year_and_month_match() {
        // Arrange
        let earlier = Date::new(1, 6, 2026).unwrap();
        let later = Date::new(2, 6, 2026).unwrap();

        // Act
        let is_earlier_less = earlier < later;

        // Assert
        assert!(is_earlier_less);
    }

    /// Verifies array organization methods successfully sort dynamic dates sequentially.
    #[test]
    fn ordering_sorts_a_vec_of_dates_chronologically() {
        // Arrange
        let mut dates = [
            Date::new(15, 6, 2026).unwrap(),
            Date::new(1, 1, 2025).unwrap(),
            Date::new(1, 1, 2026).unwrap(),
        ];

        // Act
        dates.sort();

        // Assert
        assert_eq!(
            dates.iter().map(|d| d.year).collect::<Vec<_>>(),
            vec![2025, 2026, 2026]
        );
    }

    // endregion
}

// endregion
