//! Although Element struct may contains [`Py<Element>`] as children, it is not
//! possible to create cyclic references because we don't allow mutate the
//! children after creation.

// TODO: remove this
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod executor;
pub mod pulse;
pub mod quant;
pub mod schedule;
pub mod shape;
pub mod util;

use num::Complex;

type Complex64 = Complex<f64>;
