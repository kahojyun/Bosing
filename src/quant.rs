use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
    sync::Arc,
};

use anyhow::{bail, Result};
use num::NumCast;
use numpy::Complex64;
use ordered_float::NotNan;
use pyo3::{prelude::*, types::PyFloat, IntoPy};

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
    ($t:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
        pub struct $t(NotNan<f64>);

        impl $t {
            pub fn new(value: f64) -> Result<Self> {
                Ok(Self(NotNan::new(value)?))
            }

            pub fn value(&self) -> f64 {
                self.0.into_inner()
            }

            pub const INFINITY: Self = Self(unsafe { NotNan::new_unchecked(f64::INFINITY) });
            pub const ZERO: Self = Self(unsafe { NotNan::new_unchecked(0.0) });
        }

        impl<'py> FromPyObject<'py> for $t {
            fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
                let value = ob.extract()?;
                Ok(Self::new(value)?)
            }
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
                Self(self.0 * rhs)
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
                Self(self.0 / rhs)
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

        impl IntoPy<PyObject> for $t {
            fn into_py(self, py: Python) -> PyObject {
                PyFloat::new_bound(py, self.value()).into()
            }
        }

        impl From<$t> for f64 {
            fn from(q: $t) -> Self {
                q.value()
            }
        }

        impl TryFrom<f64> for $t {
            type Error = anyhow::Error;

            fn try_from(value: f64) -> Result<Self> {
                Self::new(value)
            }
        }
    };
}

impl_quant!(Time);
impl_quant!(Frequency);
impl_quant!(Phase);
impl_quant!(Amplitude);

impl Phase {
    fn radians(&self) -> f64 {
        self.value() * std::f64::consts::TAU
    }

    pub fn phaser(&self) -> Complex64 {
        Complex64::from_polar(1.0, self.radians())
    }
}

impl Frequency {
    pub fn dt(&self) -> Time {
        Time::new(1.0 / self.value()).expect("Frequency should be non-zero")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AlignedIndex(NotNan<f64>);

impl AlignedIndex {
    pub fn new(time: Time, sample_rate: Frequency, align_level: i32) -> Result<Self> {
        fn scaleb(x: f64, s: i32) -> f64 {
            x * (s as f64).exp2()
        }
        let scaled_sr = scaleb(sample_rate.value(), -align_level);
        let i = (time.value() * scaled_sr).ceil();
        let aligned_index = scaleb(i, align_level);
        Self::from_value(aligned_index)
    }

    fn from_value(value: f64) -> Result<Self> {
        if value.is_infinite() {
            bail!("Infinite value is not allowed");
        }
        Ok(Self(NotNan::new(value)?))
    }

    pub fn value(&self) -> f64 {
        self.0.into_inner()
    }

    pub fn ceil_as_usize(&self) -> Option<usize> {
        <usize as NumCast>::from(self.0.ceil())
    }

    pub fn index_offset(&self) -> Result<Self> {
        Self::from_value(self.0.ceil() - self.0.into_inner())
    }
}

macro_rules! impl_id {
    ($t:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $t(Arc<str>);

        impl $t {
            pub fn new(name: impl Into<Arc<str>>) -> Self {
                Self(name.into())
            }
        }

        impl<'py> FromPyObject<'py> for $t {
            fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
                let name = ob.extract::<&str>()?;
                Ok(Self::new(name))
            }
        }

        impl IntoPy<PyObject> for $t {
            fn into_py(self, py: Python) -> PyObject {
                self.0.into_py(py)
            }
        }

        impl<'a> IntoPy<PyObject> for &'a $t {
            fn into_py(self, py: Python) -> PyObject {
                self.0.to_object(py)
            }
        }
    };
}

impl_id!(ChannelId);
impl_id!(ShapeId);
