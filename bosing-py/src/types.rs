//! Wrappers for the types used in the Bosing API.

use std::{convert::Infallible, fmt};

use bosing::quant;
use pyo3::{
    Borrowed,
    exceptions::PyValueError,
    prelude::*,
    pybacked::PyBackedStr,
    types::{PyFloat, PyString},
};

macro_rules! wrap_value {
    ($wrapper:ident, $inner:ty) => {
        /// Wrapper for a domain value.
        #[derive(Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $wrapper(pub $inner);

        impl $wrapper {
            pub const ZERO: Self = Self(<$inner>::ZERO);

            pub fn new(value: f64) -> Result<Self, quant::Error> {
                <$inner>::new(value).map(Self)
            }
        }

        impl<'a, 'py> FromPyObject<'a, 'py> for $wrapper {
            type Error = PyErr;

            fn extract(ob: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
                let value = ob.extract::<f64>()?;
                <$inner>::new(value)
                    .map_err(|e| PyValueError::new_err(format!("Invalid value. Error: {e}")))
                    .map(Self)
            }
        }

        impl<'py> IntoPyObject<'py> for $wrapper {
            type Target = PyFloat;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                self.0.value().into_pyobject(py)
            }
        }

        impl<'a, 'py> IntoPyObject<'py> for &'a $wrapper {
            type Target = PyFloat;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                self.0.value().into_pyobject(py)
            }
        }

        impl From<$wrapper> for $inner {
            fn from(value: $wrapper) -> Self {
                value.0
            }
        }

        impl From<$inner> for $wrapper {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }
    };
}

wrap_value!(Frequency, quant::Frequency);
wrap_value!(Phase, quant::Phase);
wrap_value!(Time, quant::Time);
wrap_value!(Amplitude, quant::Amplitude);

impl Time {
    pub const INFINITY: Self = Self(quant::Time::INFINITY);
}

macro_rules! wrap_id {
    ($wrapper:ident, $inner:ty) => {
        /// Wrapper for an ID type.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct $wrapper(pub $inner);

        impl<'a, 'py> FromPyObject<'a, 'py> for $wrapper {
            type Error = PyErr;

            fn extract(ob: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
                let s = ob.extract::<PyBackedStr>()?;
                Ok(Self(<$inner>::new(&*s)))
            }
        }

        impl<'py> IntoPyObject<'py> for $wrapper {
            type Target = PyString;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                self.0.0.into_pyobject(py)
            }
        }

        impl<'a, 'py> IntoPyObject<'py> for &'a $wrapper {
            type Target = PyString;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                self.0.0.into_pyobject(py)
            }
        }

        impl From<$wrapper> for $inner {
            fn from(value: $wrapper) -> Self {
                value.0
            }
        }

        impl From<$inner> for $wrapper {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl fmt::Display for $wrapper {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

wrap_id!(ChannelId, quant::ChannelId);
wrap_id!(ShapeId, quant::ShapeId);
wrap_id!(Label, quant::Label);
