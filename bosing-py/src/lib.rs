//! Python bindings for the bosing library.
mod extract;
mod plot;
mod repr;
mod types;

use pyo3::prelude::*;

/// Export the bosing library to Python.
#[pymodule]
pub mod _bosing {
    #[pymodule_export]
    use crate::{
        elements::{
            Absolute, AbsoluteEntry, Alignment, Barrier, Direction, Element, Grid, GridEntry,
            GridLength, GridLengthUnit, Play, Repeat, SetFreq, SetPhase, ShiftFreq, ShiftPhase,
            Stack, SwapPhase,
        },
        plot::{Args, Item, ItemKind},
        shapes::{Hann, Interp, Shape},
        wavegen::{generate_waveforms, generate_waveforms_with_states, Channel, OscState},
    };
}

macro_rules! push_repr {
    ($vec:expr, $py:expr, $value:expr) => {
        $vec.push(crate::repr::Arg::positional($value, $py));
    };
    ($vec:expr, $py:expr, $key:expr, $value:expr) => {
        $vec.push(crate::repr::Arg::keyword(
            pyo3::intern!($py, $key).clone().unbind(),
            $value,
            $py,
        ));
    };
    ($vec:expr, $py:expr, $key:expr, $value:expr, $default:expr) => {
        $vec.push(crate::repr::Arg::key_with_default(
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
