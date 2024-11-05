mod extract;
mod plot;
mod repr;

use plot::ItemKind;
use pyo3::prelude::*;

use elements::{
    Absolute, AbsoluteEntry, Barrier, Element, Grid, GridEntry, Play, Repeat, SetFreq, SetPhase,
    ShiftFreq, ShiftPhase, Stack, SwapPhase,
};
use shapes::{Hann, Interp, Shape};
use wavegen::{generate_waveforms, generate_waveforms_with_states, Channel, OscState};

pub(crate) use elements::{Alignment, Direction, GridLength, GridLengthUnit};

#[pymodule]
fn _bosing(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Absolute>()?;
    m.add_class::<AbsoluteEntry>()?;
    m.add_class::<Alignment>()?;
    m.add_class::<Barrier>()?;
    m.add_class::<Channel>()?;
    m.add_class::<Direction>()?;
    m.add_class::<Element>()?;
    m.add_class::<Grid>()?;
    m.add_class::<GridEntry>()?;
    m.add_class::<GridLength>()?;
    m.add_class::<GridLengthUnit>()?;
    m.add_class::<Hann>()?;
    m.add_class::<Interp>()?;
    m.add_class::<Play>()?;
    m.add_class::<Repeat>()?;
    m.add_class::<SetFreq>()?;
    m.add_class::<SetPhase>()?;
    m.add_class::<ShiftFreq>()?;
    m.add_class::<ShiftPhase>()?;
    m.add_class::<Shape>()?;
    m.add_class::<Stack>()?;
    m.add_class::<SwapPhase>()?;
    m.add_class::<OscState>()?;
    m.add_function(wrap_pyfunction!(generate_waveforms, m)?)?;
    m.add_function(wrap_pyfunction!(generate_waveforms_with_states, m)?)?;
    m.add_class::<ItemKind>()?;
    Ok(())
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
