//! Wrappers for the types used in the Bosing API.

use std::fmt;

use bosing::quant;
use pyo3::{exceptions::PyValueError, prelude::*};

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

        impl FromPyObject<'_> for $wrapper {
            fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
                let value = ob.extract::<f64>()?;
                <$inner>::new(value)
                    .map_err(|e| PyValueError::new_err(format!("Invalid value. Error: {e}")))
                    .map(Self)
            }
        }

        impl ToPyObject for $wrapper {
            fn to_object(&self, py: Python<'_>) -> PyObject {
                self.0.value().to_object(py)
            }
        }

        impl IntoPy<PyObject> for $wrapper {
            fn into_py(self, py: Python<'_>) -> PyObject {
                self.to_object(py)
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

        impl FromPyObject<'_> for $wrapper {
            fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
                let s = ob.extract::<String>()?;
                Ok(Self(<$inner>::new(s)))
            }
        }

        impl ToPyObject for $wrapper {
            fn to_object(&self, py: Python<'_>) -> PyObject {
                self.0 .0.to_object(py)
            }
        }

        impl IntoPy<PyObject> for $wrapper {
            fn into_py(self, py: Python<'_>) -> PyObject {
                self.to_object(py)
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
