//! Although Element struct may contains [`Py<Element>`] as children, it is not
//! possible to create cyclic references because we don't allow mutate the
//! children after creation.
use std::sync::Arc;

use hashbrown::HashMap;
use numpy::{prelude::*, AllowTypeChange, PyArray2, PyArrayLike2};
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError, PyValueError},
    prelude::*,
};

use crate::{
    executor::Executor,
    pulse::Sampler,
    quant::{Frequency, Time},
    schedule::ElementCommonBuilder,
};

mod executor;
mod pulse;
mod quant;
mod schedule;
mod shape;

/// Channel configuration.
///
/// `align_level` is the time axis alignment granularity. With sampling interval
/// :math:`\Delta t` and `align_level` :math:`n`, start of pulse is aligned to
/// the nearest multiple of :math:`2^n \Delta t`.
///
/// Args:
///     base_freq (float): Base frequency of the channel.
///     sample_rate (float): Sample rate of the channel.
///     length (int): Length of the waveform.
///     delay (float): Delay of the channel. Defaults to 0.0.
///     align_level (int): Time axis alignment granularity. Defaults to -10.
#[pyclass(get_all, frozen)]
#[derive(Debug, Clone)]
struct Channel {
    base_freq: f64,
    sample_rate: f64,
    length: usize,
    delay: f64,
    align_level: i32,
}

#[pymethods]
impl Channel {
    #[new]
    #[pyo3(signature = (base_freq, sample_rate, length, *, delay=0.0, align_level=-10))]
    fn new(base_freq: f64, sample_rate: f64, length: usize, delay: f64, align_level: i32) -> Self {
        Channel {
            base_freq,
            sample_rate,
            length,
            delay,
            align_level,
        }
    }
}

/// Alignment of a schedule element.
///
/// The alignment of a schedule element is used to align the element within its
/// parent element. The alignment can be one of the following:
///
/// - :attr:`Alignment.End`
/// - :attr:`Alignment.Start`
/// - :attr:`Alignment.Center`
/// - :attr:`Alignment.Stretch`: Stretch the element to fill the parent.
#[pyclass(frozen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alignment {
    End,
    Start,
    Center,
    Stretch,
}

#[pymethods]
impl Alignment {
    /// Convert the value to Alignment.
    ///
    /// The value can be one of the following:
    ///
    /// - :class:`Alignment`
    /// - "end"
    /// - "start"
    /// - "center"
    /// - "stretch"
    ///
    /// Args:
    ///     obj (str | Alignment): The value to convert.
    /// Returns:
    ///     Alignment: The converted value.
    /// Raises:
    ///     ValueError: If the value cannot be converted to Alignment.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(s) = obj.extract() {
            let alignment = match s {
                "end" => Some(Alignment::End),
                "start" => Some(Alignment::Start),
                "center" => Some(Alignment::Center),
                "stretch" => Some(Alignment::Stretch),
                _ => None,
            };
            if let Some(alignment) = alignment {
                return Py::new(obj.py(), alignment);
            }
        }
        let msg = concat!(
            "Failed to convert the value to Alignment. ",
            "Must be Alignment or one of 'end', 'start', 'center', 'stretch'"
        );
        Err(PyValueError::new_err(msg))
    }
}

fn extract_alignment(obj: &Bound<'_, PyAny>) -> PyResult<Alignment> {
    Alignment::convert(obj).and_then(|x| x.extract(obj.py()))
}

/// Base class for shapes.
///
/// Shapes are used to define the envelope of a pulse. Internally, the shape is
/// represented as a function :math:`f(t)` defined on the interval :math:`t \in
/// [-0.5, 0.5]`. The shape should be normalized such that :math:`f(\pm 0.5) = 0`
/// and :math:`f(0) = 1`.
///
/// Following shapes are supported:
///
/// - :class:`Hann`: Hann window.
/// - :class:`Interp`: Interpolated shape.
#[pyclass(subclass, frozen)]
#[derive(Debug, Clone)]
struct Shape;

impl Shape {
    fn get_rust_shape(slf: &Bound<'_, Shape>) -> PyResult<shape::Shape> {
        if slf.downcast::<Hann>().is_ok() {
            return Ok(shape::Shape::new_hann());
        }
        if let Ok(interp) = slf.downcast::<Interp>() {
            let interp = interp.get();
            return shape::Shape::new_interp(
                interp.knots.clone(),
                interp.controls.clone(),
                interp.degree,
            )
            .map_err(|e| PyValueError::new_err(e.to_string()));
        }
        Err(PyTypeError::new_err("Invalid shape type."))
    }
}

/// A Hann shape.
#[pyclass(extends=Shape, frozen)]
#[derive(Debug, Clone)]
struct Hann;

#[pymethods]
impl Hann {
    #[new]
    fn new() -> (Self, Shape) {
        (Self, Shape)
    }
}

/// An interpolated shape.
///
/// The interpolated shape use a B-spline. :func:`scipy.interpolate.make_interp_spline`
/// can be used to calculate the parameters.
///
/// .. caution::
///
///     It's user's responsibility to ensure the b-spline parameters are valid and
///     the shape is normalized such that :math:`f(\pm 0.5) = 0` and :math:`f(0) = 1`.
///
/// Args:
///     knots (Sequence[float]): Knots of the B-spline.
///     controls (Sequence[float]): Control points of the B-spline.
///     degree (int): Degree of the B-spline.
/// Example:
///     .. code-block:: python
///
///         import numpy as np
///         from scipy.interpolate import make_interp_spline
///         from bosing import Interp
///         x = np.linspace(0, np.pi, 10)
///         y = np.sin(x)
///         x = (x - x[0]) / (x[-1] - x[0]) - 0.5 # Normalize x to [-0.5, 0.5]
///         spline = make_interp_spline(x, y, k=3)
///         interp = Interp(spline.t, spline.c, spline.k)
#[pyclass(extends=Shape, get_all, frozen)]
#[derive(Debug, Clone)]
struct Interp {
    knots: Vec<f64>,
    controls: Vec<f64>,
    degree: usize,
}

#[pymethods]
impl Interp {
    #[new]
    fn new(knots: Vec<f64>, controls: Vec<f64>, degree: usize) -> PyResult<(Self, Shape)> {
        Ok((
            Self {
                knots,
                controls,
                degree,
            },
            Shape,
        ))
    }
}

fn extract_margin(obj: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    if let Ok(v) = obj.extract() {
        return Ok((v, v));
    }
    if let Ok((v1, v2)) = obj.extract() {
        return Ok((v1, v2));
    }
    let msg = "Failed to convert the value to (float, float).";
    Err(PyValueError::new_err(msg))
}

/// Base class for schedule elements.
///
/// A schedule element is a node in the tree structure of a schedule similar to
/// HTML elements. The design is inspired by `XAML in WPF / WinUI
/// <https://learn.microsoft.com/en-us/windows/apps/design/layout/layouts-with-xaml>`_
///
/// Every element has the following properties:
///
/// - :attr:`margin`
///     The margin of an element is a tuple of two floats representing the
///     margin before and after the element. If :attr:`margin` is set to a
///     single float, both sides use the same value.
///
///     Similar to margins in XAML, margins don't collapse. For example, if two
///     elements have a margin of 10 and 20, the space between the two elements
///     is 30, not 20.
///
/// - :attr:`alignment`
///     The alignment of the element. Currently, this property takes effect only
///     when the element is a child of a :class:`Grid` element.
///
/// - :attr:`phantom`
///     Whether the element is a phantom element. Phantom elements are measured
///     and arranged in the layout but do not add to the waveforms.
///
/// - :attr:`duration`, :attr:`max_duration`, and :attr:`min_duration`
///     Constraints on the duration of the element. When :attr:`duration`,
///     :attr:`max_duration`, and :attr:`min_duration` are conflicting, the
///     priority is as follows:
///
///     1. :attr:`min_duration`
///     2. :attr:`max_duration`
///     3. :attr:`duration`
///
///     When :attr:`duration` is not set, the duration is calculated such that
///     the element occupies the minimum duration.
///
/// There are two types of elements:
///
/// - Instruction elements:
///     Elements that instruct the waveform generator to perform certain
///     operations, such as playing a pulse or setting the phase of a channel.
///
///     - :class:`Play`: Play a pulse on a channel.
///     - :class:`ShiftPhase`: Shift the phase of a channel.
///     - :class:`SetPhase`: Set the phase of a channel.
///     - :class:`ShiftFreq`: Shift the frequency of a channel.
///     - :class:`SetFreq`: Set the frequency of a channel.
///     - :class:`SwapPhase`: Swap the phase of two channels.
///
///     The timing information required by the waveform generator is calculated
///     by the layout system.
///
/// - Layout elements:
///     Elements that control the layout of child elements.
///
///     - :class:`Grid`: Grid layout.
///     - :class:`Stack`: Stack layout.
///     - :class:`Absolute`: Absolute layout.
///     - :class:`Repeat`: Repeat element.
///     - :class:`Barrier`: Barrier element.
///
/// Args:
///     margin (float | tuple[float, float]): Margin of the element. Defaults to
///         0.
///     alignment (str | Alignment): Alignment of the element. The value can
///         be :class:`Alignment` or one of 'end', 'start', 'center', 'stretch'.
///         Defaults to :attr:`Alignment.End`.
///     phantom (bool): Whether the element is a phantom element and should not
///         add to waveforms. Defaults to ``False``.
///     duration (float): Duration of the element. Defaults to ``None``.
///     max_duration (float): Maximum duration of the element. Defaults to
///         ``inf``.
///     min_duration (float): Minimum duration of the element. Defaults to 0.
#[pyclass(subclass, frozen)]
#[derive(Debug, Clone)]
struct Element(Arc<schedule::Element>);

#[pymethods]
impl Element {
    #[getter]
    fn margin(&self) -> (f64, f64) {
        self.0.common().margin()
    }

    #[getter]
    fn alignment(&self) -> Alignment {
        self.0.common().alignment()
    }

    #[getter]
    fn phantom(&self) -> bool {
        self.0.common().phantom()
    }

    #[getter]
    fn duration(&self) -> Option<f64> {
        self.0.common().duration()
    }

    #[getter]
    fn max_duration(&self) -> f64 {
        self.0.common().max_duration()
    }

    #[getter]
    fn min_duration(&self) -> f64 {
        self.0.common().min_duration()
    }
}

fn build_element(
    variant: impl Into<schedule::ElementVariant>,
    margin: Option<&Bound<'_, PyAny>>,
    alignment: Option<&Bound<'_, PyAny>>,
    phantom: bool,
    duration: Option<f64>,
    max_duration: f64,
    min_duration: f64,
) -> PyResult<Element> {
    let mut builder = ElementCommonBuilder::new();
    if let Some(obj) = margin {
        builder.margin(extract_margin(obj)?);
    }
    if let Some(obj) = alignment {
        builder.alignment(extract_alignment(obj)?);
    }
    builder
        .phantom(phantom)
        .duration(duration)
        .max_duration(max_duration)
        .min_duration(min_duration);
    let common = builder
        .build()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(Element(Arc::new(schedule::Element::new(common, variant))))
}

/// A pulse play element.
///
/// Given the pulse envelope :math:`E(t)`, channel total frequency :math:`f_c`,
/// and channel phase :math:`\phi_c`, the the final pulse :math:`P(t)` starts at
/// :math:`t_0` with sideband will be
///
/// .. math::
///
///     E_d(t) = \left( 1 + i \alpha \frac{d}{dt} \right) E(t)
///
///     P(t) = E_d(t) \exp \big[ i 2 \pi (f_c t + f_p (t-t_0) + \phi_c + \phi_p) \big]
///
/// where :math:`\alpha` is the `drag_coef` parameter, :math:`f_p` is the
/// `frequency` parameter, and :math:`\phi_p` is the `phase` parameter. The
/// derivative is calculated using the central difference method. An exceptional
/// case is when the pulse is a rectangular pulse. In this case, the drag
/// coefficient is ignored.
///
/// If `flexible` is set to ``True``, the `plateau` parameter is ignored and the
/// actual plateau length is determined by the duration of the element.
///
/// .. caution::
///
///     The unit of phase is number of cycles, not radians. For example, a phase
///     of :math:`0.5` means a phase shift of :math:`\pi` radians.
///
/// Args:
///     channel_id (str): Target channel ID.
///     shape_id (str | None): Shape ID of the pulse. If ``None``, the pulse is
///         a rectangular pulse.
///     amplitude (float): Amplitude of the pulse.
///     width (float): Width of the pulse.
///     plateau (float): Plateau length of the pulse. Defaults to 0.
///     drag_coef (float): Drag coefficient of the pulse. If the pulse is a
///         rectangular pulse, the drag coefficient is ignored. Defaults to 0.
///     frequency (float): Additional frequency of the pulse on top of channel
///         base frequency and frequency shift. Defaults to 0.
///     phase (float): Additional phase of the pulse in **cycles**. Defaults to
///         0.
///     flexible (bool): Whether the pulse has flexible plateau length. Defaults
///         to ``False``.
#[pyclass(extends=Element, frozen)]
#[derive(Debug, Clone)]
struct Play;

#[pymethods]
impl Play {
    #[new]
    #[pyo3(signature = (
        channel_id,
        shape_id,
        amplitude,
        width,
        *,
        plateau=0.0,
        drag_coef=0.0,
        frequency=0.0,
        phase=0.0,
        flexible=false,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: String,
        shape_id: Option<String>,
        amplitude: f64,
        width: f64,
        plateau: f64,
        drag_coef: f64,
        frequency: f64,
        phase: f64,
        flexible: bool,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::Play::new(channel_id, shape_id, amplitude, width)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let variant = variant
            .with_plateau(plateau)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let variant = variant
            .with_drag_coef(drag_coef)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let variant = variant
            .with_frequency(frequency)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let variant = variant
            .with_phase(phase)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let variant = variant.with_flexible(flexible);
        Ok((
            Self,
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<String> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .channel_id();
        Ok(ret.to_string())
    }

    #[getter]
    fn shape_id(slf: &Bound<'_, Self>) -> PyResult<Option<String>> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .shape_id();
        Ok(ret.map(|x| x.to_string()))
    }

    #[getter]
    fn amplitude(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .amplitude();
        Ok(ret)
    }

    #[getter]
    fn width(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .width();
        Ok(ret)
    }

    #[getter]
    fn plateau(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .plateau();
        Ok(ret)
    }

    #[getter]
    fn drag_coef(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .drag_coef();
        Ok(ret)
    }

    #[getter]
    fn frequency(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .frequency();
        Ok(ret)
    }

    #[getter]
    fn phase(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .phase();
        Ok(ret)
    }

    #[getter]
    fn flexible(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .flexible();
        Ok(ret)
    }
}

/// A phase shift element.
///
/// Phase shift will be added to the channel phase offset :math:`\phi_c` and is
/// time-independent.
///
/// .. caution::
///
///     The unit of phase is number of cycles, not radians. For example, a phase
///     of :math:`0.5` means a phase shift of :math:`\pi` radians.
///
/// Args:
///     channel_id (str): Target channel ID.
///     phase (float): Phase shift in **cycles**.
#[pyclass(extends=Element, frozen)]
#[derive(Debug, Clone)]
struct ShiftPhase;

#[pymethods]
impl ShiftPhase {
    #[new]
    #[pyo3(signature = (
        channel_id,
        phase,
        *,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: String,
        phase: f64,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::ShiftPhase::new(channel_id, phase)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok((
            Self,
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<String> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_shift_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the shift_phase variant from the element.",
            ))?
            .channel_id();
        Ok(ret.to_string())
    }

    #[getter]
    fn phase(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_shift_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the shift_phase variant from the element.",
            ))?
            .phase();
        Ok(ret)
    }
}

/// A phase set element.
///
/// Waveform generator treats the base frequency :math:`f_0` and the channel
/// frequency shift :math:`\Delta f` differently. :math:`f_0` is never changed
/// during the execution of the schedule, while :math:`\Delta f` can be changed
/// by :class:`ShiftFreq` and :class:`SetFreq`. :class:`SetPhase` only considers
/// :math:`\Delta f` part of the frequency. The channel phase offset
/// :math:`\phi_c` will be adjusted such that
///
/// .. math:: \Delta f t + \phi_c = \phi
///
/// at the scheduled time point, where :math:`\phi` is the `phase` parameter.
///
/// .. caution::
///
///     The unit of phase is number of cycles, not radians. For example, a phase
///     of :math:`0.5` means a phase shift of :math:`\pi` radians.
///
/// Args:
///     channel_id (str): Target channel ID.
///     phase (float): Target phase value in **cycles**.
#[pyclass(extends=Element, frozen)]
#[derive(Debug, Clone)]
struct SetPhase;

#[pymethods]
impl SetPhase {
    #[new]
    #[pyo3(signature = (
        channel_id,
        phase,
        *,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: String,
        phase: f64,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::SetPhase::new(channel_id, phase)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok((
            Self,
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<String> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_set_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the set_phase variant from the element.",
            ))?
            .channel_id();
        Ok(ret.to_string())
    }

    #[getter]
    fn phase(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_set_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the set_phase variant from the element.",
            ))?
            .phase();
        Ok(ret)
    }
}

/// A frequency shift element.
///
/// Frequency shift will be added to the channel frequency shift :math:`\Delta
/// f` and the channel phase offset :math:`\phi_c` will be adjusted such that
/// the phase is continuous at the scheduled time point.
///
/// Args:
///     channel_id (str): Target channel ID.
///     frequency (float): Delta frequency.
#[pyclass(extends=Element, frozen)]
#[derive(Debug, Clone)]
struct ShiftFreq;

#[pymethods]
impl ShiftFreq {
    #[new]
    #[pyo3(signature = (
        channel_id,
        frequency,
        *,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: String,
        frequency: f64,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::ShiftFreq::new(channel_id, frequency)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok((
            Self,
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<String> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_shift_freq()
            .ok_or(PyValueError::new_err(
                "Failed to get the shift_freq variant from the element.",
            ))?
            .channel_id();
        Ok(ret.to_string())
    }

    #[getter]
    fn frequency(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_shift_freq()
            .ok_or(PyValueError::new_err(
                "Failed to get the shift_freq variant from the element.",
            ))?
            .frequency();
        Ok(ret)
    }
}

/// A frequency set element.
///
/// The channel frequency shift :math:`\Delta f` will be set to the provided
/// `frequency` parameter and the channel phase offset :math:`\phi_c` will be
/// adjusted such that the phase is continuous at the scheduled time point.
/// The channel base frequency :math:`f_0` will not be changed.
///
/// Args:
///     channel_id (str): Target channel ID.
///     frequency (float): Target frequency.
#[pyclass(extends=Element, frozen)]
#[derive(Debug, Clone)]
struct SetFreq;

#[pymethods]
impl SetFreq {
    #[new]
    #[pyo3(signature = (
        channel_id,
        frequency,
        *,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: String,
        frequency: f64,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::SetFreq::new(channel_id, frequency)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok((
            Self,
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<String> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_set_freq()
            .ok_or(PyValueError::new_err(
                "Failed to get the set_freq variant from the element.",
            ))?
            .channel_id();
        Ok(ret.to_string())
    }

    #[getter]
    fn frequency(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_set_freq()
            .ok_or(PyValueError::new_err(
                "Failed to get the set_freq variant from the element.",
            ))?
            .frequency();
        Ok(ret)
    }
}

/// A phase swap element.
///
/// Different from :class:`SetPhase` and :class:`SetFreq`, both the channel
/// base frequency :math:`f_0` and the channel frequency shift :math:`\Delta f`
/// will be considered. At the scheduled time point, the phase to be swapped
/// is calculated as
///
/// .. math:: \phi(t) = (f_0 + \Delta f) t + \phi_c
///
/// Args:
///     channel_id1 (str): Target channel ID 1.
///     channel_id2 (str): Target channel ID 2.
#[pyclass(extends=Element, frozen)]
#[derive(Debug, Clone)]
struct SwapPhase;

#[pymethods]
impl SwapPhase {
    #[new]
    #[pyo3(signature = (
        channel_id1,
        channel_id2,
        *,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id1: String,
        channel_id2: String,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::SwapPhase::new(channel_id1, channel_id2);
        Ok((
            Self,
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    #[getter]
    fn channel_id1(slf: &Bound<'_, Self>) -> PyResult<String> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_swap_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the swap_phase variant from the element.",
            ))?
            .channel_id1();
        Ok(ret.to_string())
    }

    #[getter]
    fn channel_id2(slf: &Bound<'_, Self>) -> PyResult<String> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_swap_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the swap_phase variant from the element.",
            ))?
            .channel_id2();
        Ok(ret.to_string())
    }
}

/// A barrier element.
///
/// A barrier element is a no-op element. Useful for aligning elements on
/// different channels and adding space between elements in a :class:`Stack`
/// layout.
///
/// If no channel IDs are provided, the layout system will arrange the barrier
/// element as if it occupies all channels in its parent.
///
/// Args:
///     *channel_ids (str): Channel IDs. Defaults to empty.
#[pyclass(extends=Element, frozen)]
#[derive(Debug, Clone)]
struct Barrier;

#[pymethods]
impl Barrier {
    #[new]
    #[pyo3(signature = (
        *channel_ids,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    fn new(
        channel_ids: Vec<String>,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::Barrier::new(channel_ids);
        Ok((
            Self,
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    #[getter]
    fn channel_ids(slf: &Bound<'_, Self>) -> PyResult<Vec<String>> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_barrier()
            .ok_or(PyValueError::new_err(
                "Failed to get the barrier variant from the element.",
            ))?
            .channel_ids()
            .to_vec();
        Ok(ret)
    }
}

/// A repeat element.
///
/// Repeat the child element multiple times with a spacing between repetitions.
///
/// Args:
///     child (Element): Child element to repeat.
///     count (int): Number of repetitions.
///     spacing (float): Spacing between repetitions. Defaults to 0.
#[pyclass(extends=Element, get_all, frozen)]
#[derive(Debug, Clone)]
struct Repeat {
    child: Py<Element>,
}

#[pymethods]
impl Repeat {
    #[new]
    #[pyo3(signature = (
        child,
        count,
        spacing=0.0,
        *,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        child: Py<Element>,
        count: usize,
        spacing: f64,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let rust_child = child.get().0.clone();
        let variant = schedule::Repeat::new(rust_child, count)
            .with_spacing(spacing)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok((
            Self { child },
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    #[getter]
    fn count(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_repeat()
            .ok_or(PyValueError::new_err(
                "Failed to get the repeat variant from the element.",
            ))?
            .count();
        Ok(ret)
    }

    #[getter]
    fn spacing(slf: &Bound<'_, Self>) -> PyResult<f64> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_repeat()
            .ok_or(PyValueError::new_err(
                "Failed to get the repeat variant from the element.",
            ))?
            .spacing();
        Ok(ret)
    }
}

/// Layout order in a stack layout.
///
/// A stack layout has two possible children processing orders:
///
/// - :attr:`Direction.Backward`:
///     Process children in reverse order and schedule them as late as possible.
///     This is the default order.
///
/// - :attr:`Direction.Forward`:
///     Process children in original order and schedule them as early as
///     possible.
#[pyclass(frozen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Backward,
    Forward,
}

#[pymethods]
impl Direction {
    /// Convert the value to Direction.
    ///
    /// The value can be:
    ///
    /// - :class:`Direction`
    /// - str: 'backward' or 'forward'
    ///
    /// Args:
    ///     obj (str | Direction): Value to convert.
    /// Returns:
    ///     Direction: Converted value.
    /// Raises:
    ///     ValueError: If the value cannot be converted.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(s) = obj.extract() {
            let direction = match s {
                "backward" => Some(Direction::Backward),
                "forward" => Some(Direction::Forward),
                _ => None,
            };
            if let Some(direction) = direction {
                return Py::new(obj.py(), direction);
            }
        }
        let msg = concat!(
            "Failed to convert the value to Direction. ",
            "Must be Direction or one of 'backward', 'forward'"
        );
        Err(PyValueError::new_err(msg))
    }
}

fn extract_direction(obj: &Bound<'_, PyAny>) -> PyResult<Direction> {
    Direction::convert(obj).and_then(|x| x.extract(obj.py()))
}

/// A stack layout element.
///
/// Each child element occupies some channels and has a duration. Stack layout
/// will put children as close as possible without changing the order of
/// children with common channels. Two layout orders are available:
/// :attr:`Direction.Backward` and :attr:`Direction.Forward`. The default order
/// is :attr:`Direction.Backward`.
///
/// Args:
///     *children (Element): Child elements.
///     direction (str | Direction): Layout order. Defaults to 'backward'.
#[pyclass(extends=Element, get_all, frozen)]
#[derive(Debug, Clone)]
struct Stack {
    children: Vec<Py<Element>>,
}

#[pymethods]
impl Stack {
    #[new]
    #[pyo3(signature = (
        *children,
        direction=None,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        children: Vec<Py<Element>>,
        direction: Option<&Bound<'_, PyAny>>,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let rust_children = children.iter().map(|x| x.get().0.clone()).collect();
        let variant = schedule::Stack::new().with_children(rust_children);
        let variant = if let Some(obj) = direction {
            variant.with_direction(extract_direction(obj)?)
        } else {
            variant
        };
        Ok((
            Self { children },
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    /// Create a new stack layout with different children.
    ///
    /// Using this method may be more readable than specifying children in the
    /// constructor.
    ///
    /// .. code-block:: python
    ///
    ///     stack = Stack(direction='forward').with_children(
    ///         element1,
    ///         element2,
    ///     )
    ///
    /// Args:
    ///     *children (Element): New child elements.
    /// Returns:
    ///     Stack: New stack layout.
    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<'_, Self>, children: Vec<Py<Element>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let rust_children = children.iter().map(|x| x.get().0.clone()).collect();
        let rust_base = &slf.downcast::<Element>()?.get().0;
        let common = rust_base.common().clone();
        let variant = rust_base
            .try_get_stack()
            .ok_or(PyValueError::new_err(
                "Failed to get the stack variant from the element.",
            ))?
            .clone()
            .with_children(rust_children);
        Py::new(
            py,
            (
                Self { children },
                Element(Arc::new(schedule::Element::new(common, variant))),
            ),
        )
    }

    #[getter]
    fn direction(slf: &Bound<'_, Self>) -> PyResult<Direction> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_stack()
            .ok_or(PyValueError::new_err(
                "Failed to get the stack variant from the element.",
            ))?
            .direction();
        Ok(ret)
    }
}

/// A child element with an absolute time in a absolute layout.
///
/// The time of each child element is relative to the start of the absolute
/// layout.
///
/// Args:
///     time (float): Time relative to the start of the parent element.
///     element (Element): Child element.
#[pyclass(get_all, frozen)]
#[derive(Debug, Clone)]
struct AbsoluteEntry {
    time: f64,
    element: Py<Element>,
}

#[pymethods]
impl AbsoluteEntry {
    #[new]
    fn new(time: f64, element: Py<Element>) -> PyResult<Self> {
        if !time.is_finite() {
            return Err(PyValueError::new_err("Time must be finite"));
        }
        Ok(AbsoluteEntry { time, element })
    }

    /// Convert the value to AbsoluteEntry.
    ///
    /// the value can be:
    ///
    /// - AbsoluteEntry
    /// - Element
    /// - tuple[float, Element]: Time and element.
    ///
    /// Args:
    ///     obj (AbsoluteEntry | Element | tuple[float, Element]): Value to convert.
    /// Returns:
    ///     AbsoluteEntry: Converted value.
    /// Raises:
    ///     ValueError: If the value cannot be converted.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(element) = obj.extract() {
            return Py::new(py, AbsoluteEntry::new(0.0, element)?);
        }
        if let Ok((time, element)) = obj.extract() {
            return Py::new(py, AbsoluteEntry::new(time, element)?);
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to AbsoluteEntry",
        ))
    }
}

fn extract_absolute_entry(obj: &Bound<'_, PyAny>) -> PyResult<AbsoluteEntry> {
    AbsoluteEntry::convert(obj).and_then(|x| x.extract(obj.py()))
}

/// An absolute layout element.
///
/// The child elements are arranged in absolute time. The time of each child
/// element is relative to the start of the absolute schedule. The duration of
/// the absolute schedule is the maximum end time of the child elements.
///
/// The `children` argument can be:
///
/// - AbsoluteEntry
/// - Element
/// - tuple[float, Element]: Time and element.
///
/// Args:
///     *children (AbsoluteEntry | Element | tuple[float, Element]): Child elements.
/// Example:
///     .. code-block:: python
///
///         absolute = Absolute(
///             element1,
///             (1.0, element2),
///             AbsoluteEntry(2.0, element3),
///         )
#[pyclass(extends=Element, get_all, frozen)]
#[derive(Debug, Clone)]
struct Absolute {
    children: Vec<AbsoluteEntry>,
}

#[pymethods]
impl Absolute {
    #[new]
    #[pyo3(signature = (
        *children,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python<'_>,
        children: Vec<Py<PyAny>>,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let children: Vec<AbsoluteEntry> = children
            .into_iter()
            .map(|x| extract_absolute_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                schedule::AbsoluteEntry::new(element)
                    .with_time(x.time)
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .collect::<PyResult<_>>()?;
        let variant = schedule::Absolute::new().with_children(rust_children);
        Ok((
            Self { children },
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    /// Create a new absolute schedule with different children.
    ///
    /// Using this method may be more readable than specifying children in the
    /// constructor.
    ///
    /// .. code-block:: python
    ///
    ///     absolute = Absolute(duration=50e-6).with_children(
    ///         element1,
    ///         (100e-9, element2),
    ///     )
    ///
    /// Args:
    ///     *children (AbsoluteEntry | Element | tuple[float, Element]): New
    ///         child elements.
    /// Returns:
    ///     Absolute: New absolute schedule.
    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<'_, Self>, children: Vec<Py<PyAny>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let children: Vec<_> = children
            .into_iter()
            .map(|x| extract_absolute_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                schedule::AbsoluteEntry::new(element)
                    .with_time(x.time)
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .collect::<PyResult<_>>()?;
        let rust_base = &slf.downcast::<Element>()?.get().0;
        let common = rust_base.common().clone();
        let variant = rust_base
            .try_get_absolute()
            .ok_or(PyValueError::new_err(
                "Failed to get the absolute variant from the element.",
            ))?
            .clone()
            .with_children(rust_children);
        Py::new(
            py,
            (
                Self { children },
                Element(Arc::new(schedule::Element::new(common, variant))),
            ),
        )
    }
}

/// Unit of grid length.
///
/// The unit can be:
///
/// - Seconds: Fixed length in seconds.
/// - Auto: Auto length.
/// - Star: Ratio of the remaining duration.
#[pyclass(frozen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridLengthUnit {
    Seconds,
    Auto,
    Star,
}

/// Length of a grid column.
///
/// :class:`GridLength` is used to specify the length of a grid column. The
/// length can be specified in seconds, as a fraction of the remaining duration,
/// or automatically.
#[pyclass(get_all, frozen)]
#[derive(Debug, Clone)]
struct GridLength {
    value: f64,
    unit: GridLengthUnit,
}

#[pymethods]
impl GridLength {
    /// Create an automatic grid length.
    ///
    /// Returns:
    ///     GridLength: Automatic grid length.
    #[staticmethod]
    fn auto() -> Self {
        GridLength {
            value: 0.0,
            unit: GridLengthUnit::Auto,
        }
    }

    /// Create a ratio based grid length.
    ///
    /// Args:
    ///     value (float): Ratio of the remaining duration.
    /// Returns:
    ///     GridLength: Ratio based grid length.
    #[staticmethod]
    fn star(value: f64) -> PyResult<Self> {
        if !value.is_finite() || value <= 0.0 {
            return Err(PyValueError::new_err("The value must be greater than 0."));
        }
        Ok(GridLength {
            value,
            unit: GridLengthUnit::Star,
        })
    }

    /// Create a fixed grid length.
    ///
    /// Args:
    ///     value (float): Fixed length in seconds.
    /// Returns:
    ///     GridLength: Fixed grid length.
    #[staticmethod]
    fn fixed(value: f64) -> PyResult<Self> {
        if !value.is_finite() || value < 0.0 {
            return Err(PyValueError::new_err(
                "The value must be greater than or equal to 0.",
            ));
        }
        Ok(GridLength {
            value,
            unit: GridLengthUnit::Seconds,
        })
    }

    /// Convert the value to GridLength.
    ///
    /// The value can be:
    ///
    /// - GridLength
    /// - float: Fixed length in seconds.
    /// - 'auto': Auto length.
    /// - 'x*': x stars.
    /// - 'x': Fixed length in seconds.
    /// - '*': 1 star.
    ///
    /// Args:
    ///     obj (GridLength | float | str): Value to convert.
    /// Returns:
    ///     GridLength: Converted value.
    /// Raises:
    ///     ValueError: If the value cannot be converted.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(v) = obj.extract() {
            return Py::new(py, GridLength::fixed(v)?);
        }
        if let Ok(s) = obj.extract::<&str>() {
            if s == "auto" {
                return Py::new(py, GridLength::auto());
            }
            if s == "*" {
                return Py::new(py, GridLength::star(1.0)?);
            }
            if let Some(v) = s.strip_suffix('*').and_then(|x| x.parse().ok()) {
                return Py::new(py, GridLength::star(v)?);
            }
            if let Ok(v) = s.parse() {
                return Py::new(py, GridLength::fixed(v)?);
            }
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to GridLength.",
        ))
    }
}

fn extract_grid_length(obj: &Bound<'_, PyAny>) -> PyResult<GridLength> {
    GridLength::convert(obj).and_then(|x| x.extract(obj.py()))
}

/// A child element in a grid layout.
///
/// Args:
///     element (Element): Child element.
///     column (int): Column index.
///     span (int): Column span.
#[pyclass(get_all, frozen)]
#[derive(Debug, Clone)]
struct GridEntry {
    element: Py<Element>,
    column: usize,
    span: usize,
}

#[pymethods]
impl GridEntry {
    #[new]
    #[pyo3(signature = (element, column=0, span=1))]
    fn new(element: Py<Element>, column: usize, span: usize) -> PyResult<Self> {
        if span == 0 {
            return Err(PyValueError::new_err("The span must be greater than 0."));
        }
        Ok(GridEntry {
            element,
            column,
            span,
        })
    }

    /// Convert the value to GridEntry.
    ///
    /// The value can be:
    ///
    /// - GridEntry
    /// - Element
    /// - tuple[Element, int]: Element and column.
    /// - tuple[Element, int, int]: Element, column, and span.
    ///
    /// Args:
    ///     obj (GridEntry | Element | tuple[Element, int] | tuple[Element, int, int]): Value to convert.
    /// Returns:
    ///     GridEntry: Converted value.
    /// Raises:
    ///     ValueError: If the value cannot be converted.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(element) = obj.extract() {
            return Py::new(py, GridEntry::new(element, 0, 1)?);
        }
        if let Ok((element, column)) = obj.extract() {
            return Py::new(py, GridEntry::new(element, column, 1)?);
        }
        if let Ok((element, column, span)) = obj.extract() {
            return Py::new(py, GridEntry::new(element, column, span)?);
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to GridEntry.",
        ))
    }
}

fn extract_grid_entry(obj: &Bound<'_, PyAny>) -> PyResult<GridEntry> {
    GridEntry::convert(obj).and_then(|x| x.extract(obj.py()))
}

/// A grid layout element.
///
/// A grid layout has multiple columns and each child element occupies some
/// columns. The width of each column can be specified by :class:`GridLength`,
/// which can be:
///
/// - Fixed length in seconds.
/// - Auto length:
///     The width is determined by the child element.
///
/// - Star length:
///     The width id determined by remaining duration. For example, if there
///     are two columns with 1* and 2* and the remaining duration is 300 ns,
///     the width of the columns will be 100 ns and 200 ns.
///
/// Columns length can be specified with a simplified syntax:
///
/// - 'auto': Auto length.
/// - 'x*': x stars.
/// - 'x': Fixed length in seconds.
/// - '*': 1 star.
///
/// If no columns are provided, the grid layout will have one column with '*'.
///
/// Children can be provided as:
///
/// - GridEntry
/// - Element: The column index is 0 and the span is 1.
/// - tuple[Element, int]: Element and column. The span is 1.
/// - tuple[Element, int, int]: Element, column, and span.
///
/// Args:
///     *children (GridEntry | Element | tuple[Element, int] | tuple[Element, int, int]): Child elements.
///     columns (Iterable[GridLength | float | str]): Column lengths. Defaults to ['*'].
/// Example:
///     .. code-block:: python
///
///         grid = Grid(
///             GridEntry(element1, 0, 1),
///             (element2, 1),
///             (element3, 2, 2),
///             element4,
///             columns=['auto', '1*', '2'],
///         )
#[pyclass(extends=Element, get_all, frozen)]
#[derive(Debug, Clone)]
struct Grid {
    children: Vec<GridEntry>,
}

#[pymethods]
impl Grid {
    #[new]
    #[pyo3(signature = (
        *children,
        columns=vec![],
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=f64::INFINITY,
        min_duration=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python<'_>,
        children: Vec<Py<PyAny>>,
        columns: Vec<Py<PyAny>>,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<(Self, Element)> {
        let children: Vec<_> = children
            .into_iter()
            .map(|x| extract_grid_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let columns: Vec<_> = columns
            .into_iter()
            .map(|x| extract_grid_length(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                schedule::GridEntry::new(element)
                    .with_column(x.column)
                    .with_span(x.span)
            })
            .collect();
        let variant = schedule::Grid::new()
            .with_children(rust_children)
            .with_columns(columns);
        Ok((
            Self { children },
            build_element(
                variant,
                margin,
                alignment,
                phantom,
                duration,
                max_duration,
                min_duration,
            )?,
        ))
    }

    /// Create a new grid schedule with different children.
    ///
    /// Using this method may be more readable than specifying children in the
    /// constructor.
    ///
    /// .. code-block:: python
    ///
    ///     grid = Grid(columns=['auto', '*', 'auto']).with_children(
    ///         element1,
    ///         (element2, 2),
    ///         (element3, 0, 3),
    ///     )
    ///
    /// Args:
    ///     *children (GridEntry | Element | tuple[Element, int] | tuple[Element, int, int]): New child elements.
    /// Returns:
    ///     Grid: New grid schedule.
    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<'_, Self>, children: Vec<Py<PyAny>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let children: Vec<_> = children
            .into_iter()
            .map(|x| extract_grid_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                schedule::GridEntry::new(element)
                    .with_column(x.column)
                    .with_span(x.span)
            })
            .collect();
        let rust_base = &slf.downcast::<Element>()?.get().0;
        let common = rust_base.common().clone();
        let variant = rust_base
            .try_get_grid()
            .ok_or(PyValueError::new_err(
                "Failed to get the grid variant from the element.",
            ))?
            .clone()
            .with_children(rust_children);
        Py::new(
            py,
            (
                Self { children },
                Element(Arc::new(schedule::Element::new(common, variant))),
            ),
        )
    }

    #[getter]
    fn columns(slf: &Bound<'_, Self>) -> PyResult<Vec<GridLength>> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_grid()
            .ok_or(PyValueError::new_err(
                "Failed to get the grid variant from the element.",
            ))?
            .columns()
            .to_vec();
        Ok(ret)
    }
}

/// Generate waveforms from a schedule.
///
/// Args:
///     channels (Mapping[str, Channel]): Information of the channels.
///     shapes (Mapping[str, Shape]): Shapes used in the schedule.
///     schedule (Element): Root element of the schedule.
///     time_tolerance (float): Tolerance for time comparison. Default is 1e-12.
///     amp_tolerance (float): Tolerance for amplitude comparison. Default is
///         0.1 / 2^16.
///     allow_oversize (bool): Allow oversize elements. Default is ``False``.
///     crosstalk (tuple[array_like, Sequence[str]] | None): Crosstalk matrix
///         with corresponding channel ids. Default is ``None``.
/// Returns:
///     Dict[str, numpy.ndarray]: Waveforms of the channels. The key is the
///         channel name and the value is the waveform. The shape of the
///         waveform is ``(2, length)``.
/// Raises:
///     ValueError: If some input is invalid.
///     TypeError: If some input has an invalid type.
///     RuntimeError: If waveform generation fails.
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
    time_tolerance=1e-12,
    amp_tolerance=0.1 / 2f64.powi(16),
    allow_oversize=false,
    crosstalk=None,
))]
#[allow(clippy::too_many_arguments)]
fn generate_waveforms(
    py: Python<'_>,
    channels: HashMap<String, Channel>,
    shapes: HashMap<String, Py<Shape>>,
    schedule: &Bound<'_, Element>,
    time_tolerance: f64,
    amp_tolerance: f64,
    allow_oversize: bool,
    crosstalk: Option<(PyArrayLike2<'_, f64, AllowTypeChange>, Vec<String>)>,
) -> PyResult<HashMap<String, Py<PyArray2<f64>>>> {
    if let Some((crosstalk, names)) = &crosstalk {
        if crosstalk.ndim() != 2 {
            return Err(PyValueError::new_err("Crosstalk must be a 2D array."));
        }
        if crosstalk.shape()[0] != crosstalk.shape()[1] {
            return Err(PyValueError::new_err("Crosstalk must be a square matrix."));
        }
        if crosstalk.shape()[0] != names.len() {
            return Err(PyValueError::new_err(
                "The size of the crosstalk matrix must be the same as the number of names.",
            ));
        }
    }
    let root = schedule.downcast::<Element>()?.get().0.clone();
    let measured = schedule::measure(root, f64::INFINITY);
    let arrange_options = schedule::ScheduleOptions {
        time_tolerance,
        allow_oversize,
    };
    let arranged = schedule::arrange(&measured, 0.0, measured.duration(), &arrange_options)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let mut executor = Executor::new(amp_tolerance, time_tolerance);
    for (n, c) in &channels {
        executor.add_channel(n.clone(), c.base_freq);
    }
    for (n, s) in &shapes {
        let s = s.bind(py);
        executor.add_shape(n.clone(), Shape::get_rust_shape(s)?);
    }
    executor.execute(&arranged);
    let results = executor.into_result();
    let waveforms: HashMap<String, Bound<PyArray2<f64>>> = channels
        .iter()
        .map(|(n, c)| (n.clone(), PyArray2::zeros_bound(py, (2, c.length), false)))
        .collect();
    let mut sampler = Sampler::new(results);
    for (n, c) in channels {
        // SAFETY: These arrays are just created.
        let array = unsafe { waveforms[&n].as_array_mut() };
        sampler.add_channel(
            n,
            array,
            Frequency::new(c.sample_rate).unwrap(),
            Time::new(c.delay).unwrap(),
            c.align_level,
        );
    }
    if let Some((crosstalk, names)) = &crosstalk {
        sampler.set_crosstalk(crosstalk.as_array(), names.clone());
    }
    sampler.sample(time_tolerance);
    let waveforms = waveforms
        .into_iter()
        .map(|(n, w)| (n, w.unbind()))
        .collect();
    Ok(waveforms)
}

/// Generates microwave pulses for superconducting quantum computing
/// experiments.
///
/// .. caution::
///
///     The unit of phase is number of cycles, not radians. For example, a phase
///     of :math:`0.5` means a phase shift of :math:`\pi` radians.
#[pymodule]
fn bosing(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    m.add_function(wrap_pyfunction!(generate_waveforms, m)?)?;
    Ok(())
}
