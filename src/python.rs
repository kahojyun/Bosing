//! Python bindings for the bosing library.
mod extract;
mod plot;
mod repr;

use pyo3::prelude::*;

pub(crate) use elements::{Alignment, Direction, GridLength};

/// Export the bosing library to Python.
#[pymodule(name = "_bosing")]
mod export {
    #[pymodule_export]
    use super::{
        elements::{
            Absolute, AbsoluteEntry, Alignment, Barrier, Direction, Element, Grid, GridEntry,
            GridLength, GridLengthUnit, Play, Repeat, SetFreq, SetPhase, ShiftFreq, ShiftPhase,
            Stack, SwapPhase,
        },
        plot::ItemKind,
        shapes::{Hann, Interp, Shape},
        wavegen::{generate_waveforms, generate_waveforms_with_states, Channel, OscState},
    };
}

macro_rules! push_repr {
    ($vec:expr, $py:expr, $value:expr) => {
        $vec.push(crate::python::repr::Arg::positional($value, $py));
    };
    ($vec:expr, $py:expr, $key:expr, $value:expr) => {
        $vec.push(crate::python::repr::Arg::keyword(
            pyo3::intern!($py, $key).clone().unbind(),
            $value,
            $py,
        ));
    };
    ($vec:expr, $py:expr, $key:expr, $value:expr, $default:expr) => {
        $vec.push(crate::python::repr::Arg::key_with_default(
            pyo3::intern!($py, $key).clone().unbind(),
            $value,
            $default,
            $py,
        ));
    };
}

mod elements;
mod shapes;
mod wavegen;
