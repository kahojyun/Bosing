use std::{
    fmt::Display,
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
    sync::Arc,
};

use num::cast;
use ordered_float::NotNan;
use thiserror::Error;

use crate::Complex64;

#[derive(Debug, Error)]
pub enum Error {
    #[error("NaN value is not allowed")]
    NanValue(#[from] ordered_float::FloatIsNan),
    #[error("Infinite value is not allowed")]
    InfiniteValue,
}

macro_rules! def_quant {
    ($t:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
        pub struct $t(NotNan<f64>);
    };
}

def_quant!(Time);
def_quant!(Frequency);
def_quant!(Phase);
def_quant!(Amplitude);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AlignedIndex(NotNan<f64>);

macro_rules! def_id {
    ($t:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $t(pub Arc<str>);
    };
}

def_id!(ChannelId);
def_id!(ShapeId);
def_id!(Label);

type Result<T> = std::result::Result<T, Error>;

impl Time {
    pub const INFINITY: Self = Self(unsafe { NotNan::new_unchecked(f64::INFINITY) });
}

impl Phase {
    fn radians(self) -> f64 {
        self.value() * std::f64::consts::TAU
    }

    #[must_use]
    pub fn phaser(self) -> Complex64 {
        Complex64::from_polar(1.0, self.radians())
    }
}

impl Frequency {
    #[must_use]
    pub fn dt(self) -> Time {
        Time::new(1.0 / self.value()).expect("Frequency should be non-zero")
    }
}

impl AlignedIndex {
    pub fn new(time: Time, sample_rate: Frequency, align_level: i32) -> Result<Self> {
        fn scaleb(x: f64, s: i32) -> f64 {
            let s: f64 = s.into();
            x * s.exp2()
        }
        let scaled_sr = scaleb(sample_rate.value(), -align_level);
        let i = (time.value() * scaled_sr).ceil();
        let aligned_index = scaleb(i, align_level);
        Self::from_value(aligned_index)
    }

    fn from_value(value: f64) -> Result<Self> {
        if value.is_infinite() {
            return Err(Error::InfiniteValue);
        }
        Ok(Self(NotNan::new(value)?))
    }

    #[must_use]
    pub fn value(self) -> f64 {
        self.0.into_inner()
    }

    #[must_use]
    pub fn ceil_to_usize(self) -> Option<usize> {
        cast(self.0)
    }

    #[must_use]
    pub fn index_offset(self) -> Self {
        Self::from_value(self.0.ceil() - self.0.into_inner()).expect("Should be a valid index.")
    }
}

impl Mul<Time> for Frequency {
    type Output = Phase;

    fn mul(self, rhs: Time) -> Self::Output {
        Phase::new(self.value() * rhs.value()).expect("Should be a valid phase value")
    }
}

impl Mul<Frequency> for Time {
    type Output = Phase;

    fn mul(self, rhs: Frequency) -> Self::Output {
        Phase::new(self.value() * rhs.value()).expect("Should be a valid phase value")
    }
}

macro_rules! forward_ref_binop {
    ($trait:ident, $method:ident, $t:ty) => {
        impl<'a> $trait<$t> for &'a $t {
            type Output = $t;

            fn $method(self, rhs: $t) -> Self::Output {
                $trait::$method(*self, rhs)
            }
        }

        impl<'a> $trait<&'a $t> for $t {
            type Output = $t;

            fn $method(self, rhs: &'a $t) -> Self::Output {
                $trait::$method(self, *rhs)
            }
        }

        impl<'a, 'b> $trait<&'a $t> for &'b $t {
            type Output = $t;

            fn $method(self, rhs: &'a $t) -> Self::Output {
                $trait::$method(*self, *rhs)
            }
        }
    };
}

macro_rules! impl_quant {
    ($t:ty) => {
        impl $t {
            pub fn new(value: f64) -> Result<Self> {
                Ok(Self(NotNan::new(value)?))
            }

            #[must_use]
            pub fn value(&self) -> f64 {
                self.0.into_inner()
            }

            pub const ZERO: Self = Self(unsafe { NotNan::new_unchecked(0.0) });
        }

        impl Add for $t {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        forward_ref_binop!(Add, add, $t);

        impl AddAssign for $t {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl Sub for $t {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        forward_ref_binop!(Sub, sub, $t);

        impl SubAssign for $t {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl Neg for $t {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self(-self.0)
            }
        }

        impl Mul<f64> for $t {
            type Output = Self;

            fn mul(self, rhs: f64) -> Self::Output {
                Self(NotNan::new(self.0.into_inner() * rhs).expect("result should not be NaN"))
            }
        }

        impl Mul<$t> for f64 {
            type Output = $t;

            fn mul(self, rhs: $t) -> Self::Output {
                rhs * self
            }
        }

        impl Div<f64> for $t {
            type Output = Self;

            fn div(self, rhs: f64) -> Self::Output {
                Self(NotNan::new(self.0.into_inner() / rhs).expect("result should not be NaN"))
            }
        }

        impl Sum for $t {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, Add::add)
            }
        }

        impl<'a> Sum<&'a $t> for $t {
            fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.copied().sum()
            }
        }

        impl From<$t> for f64 {
            fn from(q: $t) -> Self {
                q.value()
            }
        }

        impl TryFrom<f64> for $t {
            type Error = Error;

            fn try_from(value: f64) -> Result<Self> {
                Self::new(value)
            }
        }

        impl Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.value().fmt(f)
            }
        }
    };
}

impl_quant!(Time);
impl_quant!(Frequency);
impl_quant!(Phase);
impl_quant!(Amplitude);

macro_rules! impl_id {
    ($t:ty) => {
        impl $t {
            pub fn new(name: impl Into<Arc<str>>) -> Self {
                Self(name.into())
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

impl_id!(ChannelId);
impl_id!(ShapeId);
impl_id!(Label);
