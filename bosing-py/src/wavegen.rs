mod pyfn;

use hashbrown::HashMap;
use ndarray::ArrayViewMut2;
use numpy::{prelude::*, AllowTypeChange, PyArray2, PyArrayLike2};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};

use crate::{
    executor::{self, Executor},
    pulse::{
        apply_fir_inplace, apply_iir_inplace, apply_iq_inplace, apply_offset_inplace, List, Sampler,
    },
    quant::{Amplitude, ChannelId, Frequency, Phase, ShapeId, Time},
};

use super::{
    elements::Element,
    extract::{FirArray, IirArray, IqMatrix, OffsetArray},
    repr::{Arg, Rich},
    shapes::Shape,
};

pub use self::pyfn::{generate_waveforms, generate_waveforms_with_states};

/// Channel configuration.
///
/// `align_level` is the time axis alignment granularity. With sampling interval
/// :math:`\Delta t` and `align_level` :math:`n`, start of pulse is aligned to
/// the nearest multiple of :math:`2^n \Delta t`.
///
/// Each channel can be either real or complex. If the channel is complex, the
/// filter will be applied to both I and Q components. If the channel is real,
/// `iq_matrix` will be ignored.
///
/// .. caution::
///
///     Crosstalk matrix will not be applied to offset.
///
/// Args:
///     base_freq (float): Base frequency of the channel.
///     sample_rate (float): Sample rate of the channel.
///     length (int): Length of the waveform.
///     delay (float): Delay of the channel. Defaults to ``0.0``.
///     align_level (int): Time axis alignment granularity. Defaults to ``-10``.
///     iq_matrix (array_like[2, 2] | None): IQ matrix of the channel. Defaults
///         to ``None``.
///     offset (Sequence[float] | None): Offsets of the channel. The length of the
///         sequence should be 2 if the channel is complex, or 1 if the channel is
///         real. Defaults to ``None``.
///     iir (array_like[N, 6] | None): IIR filter of the channel. The format of
///         the array is ``[[b0, b1, b2, a0, a1, a2], ...]``, which is the same
///         as `sos` parameter of :func:`scipy.signal.sosfilt`. Defaults to ``None``.
///     fir (array_like[M] | None): FIR filter of the channel. Defaults to ``None``.
///     filter_offset (bool): Whether to apply filter to the offset. Defaults to
///         ``False``.
///     is_real (bool): Whether the channel is real. Defaults to ``False``.
#[pyclass(module = "bosing", get_all, frozen)]
#[derive(Debug, Clone)]
pub(super) struct Channel {
    base_freq: Frequency,
    sample_rate: Frequency,
    length: usize,
    delay: Time,
    align_level: i32,
    iq_matrix: Option<IqMatrix>,
    offset: Option<OffsetArray>,
    iir: Option<IirArray>,
    fir: Option<FirArray>,
    filter_offset: bool,
    is_real: bool,
}

#[pymethods]
impl Channel {
    #[new]
    #[pyo3(signature = (
        base_freq,
        sample_rate,
        length,
        *,
        delay=Time::ZERO,
        align_level=-10,
        iq_matrix=None,
        offset=None,
        iir=None,
        fir=None,
        filter_offset=false,
        is_real=false,
    ))]
    #[expect(clippy::too_many_arguments)]
    fn new(
        base_freq: Frequency,
        sample_rate: Frequency,
        length: usize,
        delay: Time,
        align_level: i32,
        iq_matrix: Option<IqMatrix>,
        offset: Option<OffsetArray>,
        iir: Option<IirArray>,
        fir: Option<FirArray>,
        filter_offset: bool,
        is_real: bool,
    ) -> PyResult<Self> {
        if is_real && iq_matrix.is_some() {
            return Err(PyValueError::new_err(
                "iq_matrix should be None when is_real==True.",
            ));
        }
        if let Some(offset) = &offset {
            let len = offset.view().dim();
            if is_real && len != 1 {
                return Err(PyValueError::new_err("is_real==True but len(shape)!=1."));
            }
            if !is_real && len != 2 {
                return Err(PyValueError::new_err("is_real==False but len(shape)!=2."));
            }
        }
        Ok(Self {
            base_freq,
            sample_rate,
            length,
            delay,
            align_level,
            iq_matrix,
            offset,
            iir,
            fir,
            filter_offset,
            is_real,
        })
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
    }
}

impl Rich for Channel {
    fn repr(slf: &Bound<'_, Self>) -> impl Iterator<Item = Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        let slf = slf.get();
        push_repr!(res, py, slf.base_freq);
        push_repr!(res, py, slf.sample_rate);
        push_repr!(res, py, slf.length);
        push_repr!(res, py, "delay", slf.delay, Time::ZERO);
        push_repr!(res, py, "align_level", slf.align_level, -10);
        // NOTE: workaround for rich issue #3531
        if let Some(iq_matrix) = &slf.iq_matrix {
            push_repr!(res, py, "iq_matrix", iq_matrix);
        }
        if let Some(offset) = &slf.offset {
            push_repr!(res, py, "offset", offset);
        }
        if let Some(iir) = &slf.iir {
            push_repr!(res, py, "iir", iir);
        }
        if let Some(fir) = &slf.fir {
            push_repr!(res, py, "fir", fir);
        }
        push_repr!(res, py, "filter_offset", slf.filter_offset, false);
        push_repr!(res, py, "is_real", slf.is_real, false);

        res.into_iter()
    }
}

/// State of a channel oscillator.
///
/// Args:
///     base_freq (float): Base frequency of the oscillator.
///     delta_freq (float): Frequency shift of the oscillator.
///     phase (float): Phase of the oscillator in **cycles**.
#[pyclass(module = "bosing", get_all, set_all)]
#[derive(Debug, Clone, Copy)]
pub(super) struct OscState {
    base_freq: Frequency,
    delta_freq: Frequency,
    phase: Phase,
}

#[pymethods]
impl OscState {
    #[new]
    const fn new(base_freq: Frequency, delta_freq: Frequency, phase: Phase) -> Self {
        Self {
            base_freq,
            delta_freq,
            phase,
        }
    }

    /// Calculate the total frequency of the oscillator.
    ///
    /// Returns:
    ///     float: Total frequency of the oscillator.
    fn total_freq(&self) -> Frequency {
        executor::OscState::from(*self).total_freq()
    }

    /// Calculate the phase of the oscillator at a given time.
    ///
    /// Args:
    ///     time (float): Time.
    ///
    /// Returns:
    ///     float: Phase of the oscillator in **cycles**.
    fn phase_at(&self, time: Time) -> Phase {
        executor::OscState::from(*self).phase_at(time)
    }

    /// Get a new state with a time shift.
    ///
    /// Args:
    ///     time (float): Time shift.
    ///
    /// Returns:
    ///     OscState: The new state.
    fn with_time_shift(&self, time: Time) -> Self {
        executor::OscState::from(*self).with_time_shift(time).into()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
    }
}

impl Rich for OscState {
    fn repr(slf: &Bound<'_, Self>) -> impl Iterator<Item = Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        let slf = slf.borrow();
        push_repr!(res, py, slf.base_freq);
        push_repr!(res, py, slf.delta_freq);
        push_repr!(res, py, slf.phase);
        res.into_iter()
    }
}

impl From<executor::OscState> for OscState {
    fn from(osc: executor::OscState) -> Self {
        Self {
            base_freq: osc.base_freq,
            delta_freq: osc.delta_freq,
            phase: osc.phase,
        }
    }
}

impl From<OscState> for executor::OscState {
    fn from(osc: OscState) -> Self {
        Self {
            base_freq: osc.base_freq,
            delta_freq: osc.delta_freq,
            phase: osc.phase,
        }
    }
}

type ChannelWaveforms = HashMap<ChannelId, Py<PyArray2<f64>>>;
type ChannelStates = HashMap<ChannelId, Py<OscState>>;
type ChannelPulses = HashMap<ChannelId, List>;
type CrosstalkMatrix<'a> = (PyArrayLike2<'a, f64, AllowTypeChange>, Vec<ChannelId>);

fn build_pulse_lists(
    schedule: &Bound<'_, Element>,
    channels: &HashMap<ChannelId, Channel>,
    shapes: &HashMap<ShapeId, Py<Shape>>,
    time_tolerance: Time,
    amp_tolerance: Amplitude,
    allow_oversize: bool,
    states: Option<&ChannelStates>,
) -> PyResult<(ChannelPulses, ChannelStates)> {
    let py = schedule.py();
    let mut executor = Executor::new(amp_tolerance, time_tolerance, allow_oversize);
    for (n, c) in channels {
        let osc = match &states {
            Some(states) => states
                .get(n)
                .ok_or_else(|| PyValueError::new_err(format!("No state for channel: {n}")))?
                .extract::<OscState>(py)?
                .into(),
            None => executor::OscState::new(c.base_freq),
        };
        executor.add_channel(n.clone(), osc);
    }
    for (n, s) in shapes {
        let s = s.bind(py);
        executor.add_shape(n.clone(), Shape::get_rust_shape(s)?);
    }
    let schedule = &schedule.get().0;
    let (states, pulselists) = py
        .allow_threads(|| {
            executor.execute(schedule)?;
            let states = executor.states();
            let pulselists = executor.into_result();
            Ok((states, pulselists))
        })
        .map_err(|e: executor::Error| PyRuntimeError::new_err(e.to_string()))?;
    let states = states
        .into_iter()
        .map(|(n, s)| Ok((n, Py::new(py, OscState::from(s))?)))
        .collect::<PyResult<_>>()?;

    Ok((pulselists, states))
}

fn sample_waveform(
    py: Python<'_>,
    channels: &HashMap<ChannelId, Channel>,
    pulse_lists: ChannelPulses,
    crosstalk: Option<CrosstalkMatrix<'_>>,
    time_tolerance: Time,
) -> PyResult<ChannelWaveforms> {
    let waveforms: HashMap<_, _> = channels
        .iter()
        .map(|(n, c)| {
            let n_w = if c.is_real { 1 } else { 2 };
            (
                n.clone(),
                PyArray2::zeros_bound(py, (n_w, c.length), false).unbind(),
            )
        })
        .collect();
    let mut sampler = Sampler::new(pulse_lists);
    for (n, c) in channels {
        // SAFETY: These arrays are just created.
        let array = unsafe { waveforms[n].bind(py).as_array_mut() };
        sampler.add_channel(n.clone(), array, c.sample_rate, c.delay, c.align_level);
    }
    if let Some((ref crosstalk, names)) = crosstalk {
        sampler.set_crosstalk(crosstalk.as_array(), names);
    }
    py.allow_threads(|| sampler.sample(time_tolerance))?;
    Ok(waveforms)
}

fn post_process(w: &mut ArrayViewMut2<'_, f64>, c: &Channel) {
    let iq_matrix = c.iq_matrix.as_ref().map(IqMatrix::view);
    let offset = c.offset.as_ref().map(OffsetArray::view);
    let iir = c.iir.as_ref().map(IirArray::view);
    let fir = c.fir.as_ref().map(FirArray::view);
    if let Some(iq_matrix) = iq_matrix {
        apply_iq_inplace(w, iq_matrix);
    }
    if c.filter_offset {
        if let Some(offset) = offset {
            apply_offset_inplace(w, offset);
        }
        if let Some(iir) = iir {
            apply_iir_inplace(w, iir);
        }
        if let Some(fir) = fir {
            apply_fir_inplace(w, fir);
        }
    } else {
        if let Some(iir) = iir {
            apply_iir_inplace(w, iir);
        }
        if let Some(fir) = fir {
            apply_fir_inplace(w, fir);
        }
        if let Some(offset) = offset {
            apply_offset_inplace(w, offset);
        }
    }
}
