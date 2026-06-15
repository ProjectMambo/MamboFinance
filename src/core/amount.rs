use crate::core::Currency;
use crate::define_struct;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Rem, Sub};

define_struct!(
Amount has {
    crate::core | currency: Currency,
} with {
    value: i64,
});

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let sign = if self.value < 0 { "Out" } else { "In" };
        let val_abs = self.value.abs();
        write!(
            f,
            "{} {}.{} | {}",
            self.currency,
            val_abs / 100,
            val_abs % 100,
            sign,
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

        Self::new(self.currency.clone(), self.value + other.value).expect("Failed to add amount")
    }
}

impl Sub for Amount {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if self.currency != other.currency {
            return self;
        }

        Self::new(self.currency.clone(), self.value - other.value)
            .expect("Failed to subtract amount")
    }
}

macro_rules! impl_mul_div_rem {
    ($($type:ty),+ $(,)?) => {
        $(
            impl Mul<$type> for Amount {
                type Output = Self;

                fn mul(self, rhs: $type) -> Amount {
                    Self::new(self.currency.clone(), self.value*(rhs as i64) ).expect("Failed to multiple amount")
                }
            }

            impl Div<$type> for Amount {
                type Output = Self;

                fn div(self, rhs: $type) -> Amount {
                    Self::new(self.currency.clone(), self.value/(rhs as i64) ).expect("Failed to divide amount")
                }
            }

            impl Rem<$type> for Amount {
                type Output = Self;

                fn rem(self, rhs: $type) -> Amount {
                    Self::new(self.currency.clone(), self.value%(rhs as i64) ).expect("Failed to modulo amount")
                }
            }
        )+
    };
}
impl_mul_div_rem!(i8, i16, i32, i64, u8, u16, u32, u64, usize);
