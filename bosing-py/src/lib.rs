//! Python bindings for the bosing library.
#![allow(clippy::doc_markdown)]
mod elements;
mod extract;
mod macros;
mod plot;
mod repr;
mod shapes;
mod types;
mod wavegen;

/// Export the bosing library to Python.
#[pyo3::pymodule]
pub mod _bosing {
    #[pymodule_export]
    pub use crate::{
        elements::{
            Absolute, AbsoluteEntry, Alignment, Barrier, Direction, Element, Grid, GridEntry,
            GridLength, GridLengthUnit, Play, Repeat, SetFreq, SetPhase, ShiftFreq, ShiftPhase,
            Stack, SwapPhase,
        },
        plot::{Args, Item, ItemKind},
        shapes::{Hann, Interp, Shape},
        wavegen::{Channel, OscState, generate_waveforms, generate_waveforms_with_states},
    };
}
