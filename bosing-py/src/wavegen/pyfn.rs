#![expect(
    clippy::redundant_pub_crate,
    reason = "pyfunction proc-macro triggers this lint"
)]

use hashbrown::HashMap;
use numpy::prelude::*;
use pyo3::{exceptions::PyValueError, prelude::*};
use rayon::prelude::*;

use crate::{
    elements::Element,
    shapes::Shape,
    types::{Amplitude, ChannelId, ShapeId, Time},
};

use super::{
    build_pulse_lists, post_process, sample_waveform, Channel, ChannelStates, ChannelWaveforms,
    CrosstalkMatrix,
};

fn default_time_tolerance() -> Time {
    Time::new(1e-12).expect("Should be valid static value.")
}

fn default_amp_tolerance() -> Amplitude {
    Amplitude::new(0.1 / 2f64.powi(16)).expect("Should be valid static value.")
}

/// Generate waveforms from a schedule.
///
/// .. caution::
///
///     Crosstalk matrix will not be applied to offset of the channels.
///
/// Args:
///     channels (Mapping[str, Channel]): Information of the channels.
///     shapes (Mapping[str, Shape]): Shapes used in the schedule.
///     schedule (Element): Root element of the schedule.
///     time_tolerance (float): Tolerance for time comparison. Default is ``1e-12``.
///     amp_tolerance (float): Tolerance for amplitude comparison. Default is
///         ``0.1 / 2**16``.
///     allow_oversize (bool): Allow elements to occupy a longer duration than
///         available. Default is ``False``.
///     crosstalk (tuple[array_like, Sequence[str]] | None): Crosstalk matrix
///         with corresponding channel ids. Default is ``None``.
///
/// Returns:
///     dict[str, numpy.ndarray]: Waveforms of the channels. The key is the
///         channel name and the value is the waveform. The shape of the
///         waveform is ``(n, length)``, where ``n`` is 2 for complex waveform
///         and 1 for real waveform.
///
/// Raises:
///     ValueError: If some input is invalid.
///     TypeError: If some input has an invalid type.
///     RuntimeError: If waveform generation fails.
///
/// Example:
///     .. code-block:: python
///
///         from bosing import Barrier, Channel, Hann, Play, Stack, generate_waveforms
///         channels = {"xy": Channel(30e6, 2e9, 1000)}
///         shapes = {"hann": Hann()}
///         schedule = Stack(duration=500e-9).with_children(
///             Play(
///                 channel_id="xy",
///                 shape_id="hann",
///                 amplitude=0.3,
///                 width=100e-9,
///                 plateau=200e-9,
///             ),
///             Barrier(duration=10e-9),
///         )
///         result = generate_waveforms(channels, shapes, schedule)
#[pyfunction]
#[pyo3(signature = (
channels,
shapes,
schedule,
*,
time_tolerance=default_time_tolerance(),
amp_tolerance=default_amp_tolerance(),
allow_oversize=false,
crosstalk=None,
))]
#[expect(clippy::too_many_arguments)]
pub fn generate_waveforms(
    py: Python<'_>,
    channels: HashMap<ChannelId, Channel>,
    shapes: HashMap<ShapeId, Py<Shape>>,
    schedule: &Bound<'_, Element>,
    time_tolerance: Time,
    amp_tolerance: Amplitude,
    allow_oversize: bool,
    crosstalk: Option<CrosstalkMatrix<'_>>,
) -> PyResult<ChannelWaveforms> {
    let (waveforms, _) = generate_waveforms_with_states(
        py,
        channels,
        shapes,
        schedule,
        time_tolerance,
        amp_tolerance,
        allow_oversize,
        crosstalk,
        None,
    )?;
    Ok(waveforms)
}

/// Generate waveforms from a schedule with initial states.
///
/// .. caution::
///
///     Crosstalk matrix will not be applied to offset of the channels.
///
/// Args:
///     channels (collections.abc.Mapping[str, Channel]): Information of the channels.
///     shapes (Mapping[str, Shape]): Shapes used in the schedule.
///     schedule (Element): Root element of the schedule.
///     time_tolerance (float): Tolerance for time comparison. Default is ``1e-12``.
///     amp_tolerance (float): Tolerance for amplitude comparison. Default is
///         ``0.1 / 2**16``.
///     allow_oversize (bool): Allow elements to occupy a longer duration than
///         available. Default is ``False``.
///     crosstalk (tuple[array_like, Sequence[str]] | None): Crosstalk matrix
///         with corresponding channel ids. Default is ``None``.
///     states (Mapping[str, OscState] | None): Initial states of the channels.
///
/// Returns:
///     tuple[dict[str, numpy.ndarray], dict[str, OscState]]: Waveforms and final states.
///
///     Waveforms part is a dictionary mapping channel names to waveforms. The shape of the
///     waveform is ``(n, length)``, where ``n`` is 2 for complex waveform and 1 for real waveform.
///
///     States part is a dictionary mapping channel names to final states.
///
/// Raises:
///     ValueError: If some input is invalid.
///     TypeError: If some input has an invalid type.
///     RuntimeError: If waveform generation fails.
#[pyfunction]
#[pyo3(signature = (
channels,
shapes,
schedule,
*,
time_tolerance=default_time_tolerance(),
amp_tolerance=default_amp_tolerance(),
allow_oversize=false,
crosstalk=None,
states=None,
))]
#[expect(clippy::too_many_arguments)]
#[expect(clippy::needless_pass_by_value, reason = "PyO3 extractor")]
pub fn generate_waveforms_with_states(
    py: Python<'_>,
    channels: HashMap<ChannelId, Channel>,
    shapes: HashMap<ShapeId, Py<Shape>>,
    schedule: &Bound<'_, Element>,
    time_tolerance: Time,
    amp_tolerance: Amplitude,
    allow_oversize: bool,
    crosstalk: Option<CrosstalkMatrix<'_>>,
    states: Option<ChannelStates>,
) -> PyResult<(ChannelWaveforms, ChannelStates)> {
    if let Some((crosstalk, names)) = &crosstalk {
        let nl = names.len();
        if crosstalk.shape() != [nl, nl] {
            return Err(PyValueError::new_err(
                "The size of the crosstalk matrix must be the same as the number of names.",
            ));
        }
    }
    let (pulse_lists, new_states) = build_pulse_lists(
        schedule,
        &channels,
        &shapes,
        time_tolerance,
        amp_tolerance,
        allow_oversize,
        states.as_ref(),
    )?;
    let waveforms = sample_waveform(py, &channels, pulse_lists, crosstalk, time_tolerance)?;
    Ok((
        py.allow_threads(|| {
            waveforms
                .into_par_iter()
                .map(|(n, w)| {
                    Python::with_gil(|py| {
                        let w = w.bind(py);
                        let mut w = w.readwrite();
                        let mut w = w.as_array_mut();
                        let c = &channels[&n];
                        py.allow_threads(|| post_process(&mut w, c));
                    });
                    (n, w)
                })
                .collect()
        }),
        new_states,
    ))
}
