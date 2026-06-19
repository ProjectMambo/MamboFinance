use crate::user::AMOUNT_LIMIT;
use crate::user::Currency;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Clone)]
pub struct Amount {
    pub value: i64,
    pub currency: Currency,
}

impl Amount {
    pub fn new(value: i64, currency: Currency) -> Self {
        Self { value, currency }
    }

    pub fn from_row_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Amount {
            value: row.get(offset)?,
            currency: Currency::from_row_offset(row, offset + 1)?,
        })
    }

    pub fn flow(&self) -> &str {
        if self.value < 0 { "> Out" } else { "< In" }
    }
}

impl Display for Amount {
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
    fn eq(&self, other: &Self) -> bool {
        self.currency == other.currency && self.value == other.value
    }
}

impl PartialOrd for Amount {
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

    fn add(self, other: Self) -> Self {
        if self.currency != other.currency {
            return self;
        }

        Self::new(self.value + other.value, self.currency)
    }
}

impl Sub for Amount {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if self.currency != other.currency {
            return self;
        }

        Self::new(self.value - other.value, self.currency)
    }
}

macro_rules! impl_mul_div_rem {
    ($($type:ty),+ $(,)?) => {
        $(
            impl Mul<$type> for Amount {
                type Output = Self;

                fn mul(self, rhs: $type) -> Amount {
                    Self::new(self.value*(rhs as i64),self.currency)
                }
            }

            impl Div<$type> for Amount {
                type Output = Self;

                fn div(self, rhs: $type) -> Amount {
                    Self::new(self.value/(rhs as i64) ,self.currency)
                }
            }

            impl Rem<$type> for Amount {
                type Output = Self;

                fn rem(self, rhs: $type) -> Amount {
                    Self::new(self.value%(rhs as i64), self.currency)
                }
            }
        )+
    };
}
impl_mul_div_rem!(i8, i16, i32, i64, u8, u16, u32, u64, usize);

#[derive(Clone, Copy)]
pub struct RawAmount {
    pub value: i64,
}

impl RawAmount {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}
