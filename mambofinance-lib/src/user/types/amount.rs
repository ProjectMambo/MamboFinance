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
