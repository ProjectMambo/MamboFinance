// Imports from internal user module
use crate::user::AMOUNT_LIMIT;
use crate::user::Currency;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Rem, Sub};

/// Represents a currency-bound monetary value, storing the absolute amount as a scaled 64-bit integer.
#[derive(Clone)]
pub struct Amount {
    pub value: i64,
    pub currency: Currency,
}

impl Amount {
    /// Constructs a new asset `Amount` instance.
    pub fn new(value: i64, currency: Currency) -> Self {
        Self { value, currency }
    }

    /// Maps a single SQLite row to an `Amount` instance using explicit database sequence indices.
    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Amount {
            value: row.get(offset)?,
            currency: Currency::from_row_offset(row, offset + 1)?,
        })
    }

    /// Determines the direction of the transaction relative to the ledger account.
    pub fn flow(&self) -> &str {
        if self.value < 0 { "> Out" } else { "< In" }
    }
}

impl Display for Amount {
    /// Formats the monetary value, scaling the internal fractional integer structure to a decimal layout.
    ///
    /// e.g., an internal value of `1050` renders as `10.50`.
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let val_abs = self.value.abs();

        write!(
            f,
            "{:#} {:>width$}.{:0<2} {:<5}",
            self.currency,
            val_abs / 100,
            val_abs % 100,
            self.flow(),
            width = AMOUNT_LIMIT,
        )
    }
}

impl Eq for Amount {}

impl PartialEq for Amount {
    /// Asserts equality only if both the tracking asset currency and numeric values match exactly.
    fn eq(&self, other: &Self) -> bool {
        self.currency == other.currency && self.value == other.value
    }
}

impl PartialOrd for Amount {
    /// Compares two amounts, returning `None` if they use differing tracking asset currencies.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.currency == other.currency {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
    }
}

impl Add for Amount {
    type Output = Self;

    /// Adds two asset amounts.
    ///
    /// # Safety / Invariants
    ///
    /// Silently returns the left-hand operand unchanged if the asset currencies do not match.
    fn add(self, other: Self) -> Self {
        if self.currency != other.currency {
            return self;
        }

        Self::new(self.value + other.value, self.currency)
    }
}

impl Sub for Amount {
    type Output = Self;

    /// Subtracts two asset amounts.
    ///
    /// # Safety / Invariants
    ///
    /// Silently returns the left-hand operand unchanged if the asset currencies do not match.
    fn sub(self, other: Self) -> Self {
        if self.currency != other.currency {
            return self;
        }

        Self::new(self.value - other.value, self.currency)
    }
}

// Macro helper to quickly generate standard arithmetic scalar operations for all native integer primitive types.
macro_rules! impl_mul_div_rem {
    ($($type:ty),+ $(,)?) => {
        $(
            impl Mul<$type> for Amount {
                type Output = Self;

                fn mul(self, rhs: $type) -> Amount {
                    Self::new(self.value * (rhs as i64), self.currency)
                }
            }

            impl Div<$type> for Amount {
                type Output = Self;

                fn div(self, rhs: $type) -> Amount {
                    Self::new(self.value / (rhs as i64), self.currency)
                }
            }

            impl Rem<$type> for Amount {
                type Output = Self;

                fn rem(self, rhs: $type) -> Amount {
                    Self::new(self.value % (rhs as i64), self.currency)
                }
            }
        )+
    };
}
impl_mul_div_rem!(i8, i16, i32, i64, u8, u16, u32, u64, usize);

/// Wrapper container providing un-allocated numeric value representations.
#[derive(Clone, Copy)]
pub struct RawAmount {
    pub value: i64,
}

impl RawAmount {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}

// region: Test

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::Label;
    use rusqlite::Connection;

    // region: helpers

    // Builds a Currency directly via its public fields, bypassing the database,
    // since Currency exposes no standalone constructor besides row-mapping.
    fn make_currency(name: &str) -> Currency {
        Currency {
            label: Label::new(name, None),
            count: 0,
        }
    }

    // endregion

    // region: Amount::new

    #[test]
    fn new_stores_value_and_currency() {
        // Arrange
        let currency = make_currency("USD");

        // Act
        let amount = Amount::new(1050, currency.clone());

        // Assert
        assert_eq!(amount.value, 1050);
        assert_eq!(amount.currency, currency);
    }

    // endregion

    // region: Amount::from_row_offset

    #[test]
    fn from_row_offset_maps_value_and_currency_columns() {
        // Arrange
        let conn = Connection::open_in_memory().expect("failed to open in-memory db");
        conn.execute(
            "CREATE TABLE amounts (value INTEGER, cur_id BLOB, cur_name TEXT, cur_count INTEGER);",
            (),
        )
        .expect("failed to create table");
        let cur_id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO amounts (value, cur_id, cur_name, cur_count) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![2599i64, cur_id, "US Dollar", 0i64],
        )
        .expect("failed to insert row");

        // Act
        let amount: Amount = conn
            .query_row(
                "SELECT value, cur_id, cur_name, cur_count FROM amounts",
                (),
                |row| Amount::from_row_offset(row, 0),
            )
            .expect("query should succeed");

        // Assert
        assert_eq!(amount.value, 2599);
        assert_eq!(amount.currency.label.name, "US Dollar");
    }

    // endregion

    // region: Amount::flow

    #[test]
    fn flow_reports_outgoing_for_negative_value() {
        // Arrange
        let amount = Amount::new(-500, make_currency("USD"));

        // Act
        let direction = amount.flow();

        // Assert
        assert_eq!(direction, "> Out");
    }

    #[test]
    fn flow_reports_incoming_for_positive_value() {
        // Arrange
        let amount = Amount::new(500, make_currency("USD"));

        // Act
        let direction = amount.flow();

        // Assert
        assert_eq!(direction, "< In");
    }

    #[test]
    fn flow_reports_incoming_for_zero_value() {
        // Arrange
        let amount = Amount::new(0, make_currency("USD"));

        // Act
        let direction = amount.flow();

        // Assert
        assert_eq!(direction, "< In");
    }

    // endregion

    // region: Display for Amount

    #[test]
    fn display_renders_scaled_decimal_value() {
        // Arrange
        let amount = Amount::new(1050, make_currency("USD"));

        // Act
        let rendered = format!("{}", amount);

        // Assert
        assert!(rendered.contains("10.50"));
        assert!(rendered.contains("< In"));
    }

    #[test]
    fn display_renders_negative_value_as_absolute_with_out_flow() {
        // Arrange
        let amount = Amount::new(-1050, make_currency("USD"));

        // Act
        let rendered = format!("{}", amount);

        // Assert
        assert!(rendered.contains("10.50"));
        assert!(rendered.contains("> Out"));
    }

    #[test]
    fn display_left_aligns_and_zero_fills_single_digit_cents() {
        // Arrange
        // The cents segment uses `{:0<2}` (zero-fill, left-aligned), so a
        // single-digit remainder of 5 renders as "50", not "05".
        let amount = Amount::new(105, make_currency("USD")); // 105 -> 1 dollar, 5 cents remainder

        // Act
        let rendered = format!("{}", amount);

        // Assert
        assert!(rendered.contains("1.50"));
    }

    #[test]
    fn display_renders_double_digit_cents_unchanged() {
        // Arrange
        let amount = Amount::new(199, make_currency("USD")); // 1 dollar, 99 cents remainder

        // Act
        let rendered = format!("{}", amount);

        // Assert
        assert!(rendered.contains("1.99"));
    }

    // endregion

    // region: Eq / PartialEq for Amount

    #[test]
    fn equality_holds_for_same_value_and_currency() {
        // Arrange
        let currency = make_currency("USD");
        let a = Amount::new(100, currency.clone());
        let b = Amount::new(100, currency);

        // Act
        let is_equal = a == b;

        // Assert
        assert!(is_equal);
    }

    #[test]
    fn equality_fails_for_different_values() {
        // Arrange
        let currency = make_currency("USD");
        let a = Amount::new(100, currency.clone());
        let b = Amount::new(200, currency);

        // Act
        let is_equal = a == b;

        // Assert
        assert!(!is_equal);
    }

    #[test]
    fn equality_fails_for_different_currencies_even_with_same_value() {
        // Arrange
        let a = Amount::new(100, make_currency("USD"));
        let b = Amount::new(100, make_currency("EUR"));

        // Act
        let is_equal = a == b;

        // Assert
        assert!(!is_equal);
    }

    // endregion

    // region: PartialOrd for Amount

    #[test]
    fn partial_cmp_orders_same_currency_amounts_by_value() {
        // Arrange
        let currency = make_currency("USD");
        let smaller = Amount::new(100, currency.clone());
        let larger = Amount::new(200, currency);

        // Act
        let is_less = smaller < larger;

        // Assert
        assert!(is_less);
    }

    #[test]
    fn partial_cmp_returns_none_for_mismatched_currencies() {
        // Arrange
        let a = Amount::new(100, make_currency("USD"));
        let b = Amount::new(200, make_currency("EUR"));

        // Act
        let result = a.partial_cmp(&b);

        // Assert
        assert_eq!(result, None);
    }

    // endregion

    // region: Add for Amount

    #[test]
    fn add_sums_values_for_matching_currencies() {
        // Arrange
        let currency = make_currency("USD");
        let a = Amount::new(100, currency.clone());
        let b = Amount::new(50, currency);

        // Act
        let sum = a + b;

        // Assert
        assert_eq!(sum.value, 150);
    }

    #[test]
    fn add_returns_left_operand_unchanged_for_mismatched_currencies() {
        // Arrange
        let a = Amount::new(100, make_currency("USD"));
        let b = Amount::new(50, make_currency("EUR"));

        // Act
        let result = a.clone() + b;

        // Assert
        assert_eq!(result.value, a.value);
        assert_eq!(result.currency, a.currency);
    }

    // endregion

    // region: Sub for Amount

    #[test]
    fn sub_subtracts_values_for_matching_currencies() {
        // Arrange
        let currency = make_currency("USD");
        let a = Amount::new(100, currency.clone());
        let b = Amount::new(30, currency);

        // Act
        let diff = a - b;

        // Assert
        assert_eq!(diff.value, 70);
    }

    #[test]
    fn sub_returns_left_operand_unchanged_for_mismatched_currencies() {
        // Arrange
        let a = Amount::new(100, make_currency("USD"));
        let b = Amount::new(30, make_currency("EUR"));

        // Act
        let result = a.clone() - b;

        // Assert
        assert_eq!(result.value, a.value);
        assert_eq!(result.currency, a.currency);
    }

    // endregion

    // region: Mul / Div / Rem (macro-generated scalar ops) for Amount

    #[test]
    fn mul_scales_value_by_integer_scalar() {
        // Arrange
        let amount = Amount::new(100, make_currency("USD"));

        // Act
        let result = amount * 3i64;

        // Assert
        assert_eq!(result.value, 300);
    }

    #[test]
    fn mul_works_with_unsigned_scalar_type() {
        // Arrange
        let amount = Amount::new(100, make_currency("USD"));

        // Act
        let result = amount * 2u8;

        // Assert
        assert_eq!(result.value, 200);
    }

    #[test]
    fn div_divides_value_by_integer_scalar() {
        // Arrange
        let amount = Amount::new(100, make_currency("USD"));

        // Act
        let result = amount / 4i64;

        // Assert
        assert_eq!(result.value, 25);
    }

    #[test]
    fn rem_returns_remainder_of_value_by_scalar() {
        // Arrange
        let amount = Amount::new(107, make_currency("USD"));

        // Act
        let result = amount % 10i64;

        // Assert
        assert_eq!(result.value, 7);
    }

    // endregion

    // region: RawAmount::new

    #[test]
    fn raw_amount_new_stores_the_given_value() {
        // Arrange
        // Act
        let raw = RawAmount::new(4242);

        // Assert
        assert_eq!(raw.value, 4242);
    }

    #[test]
    fn raw_amount_new_supports_negative_values() {
        // Arrange
        // Act
        let raw = RawAmount::new(-999);

        // Assert
        assert_eq!(raw.value, -999);
    }

    // endregion
}

// endregion
