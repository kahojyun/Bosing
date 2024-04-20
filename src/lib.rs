//! Although Element struct may contains [`Py<Element>`] as children, it is not
//! possible to create cyclic references because we don't allow mutate the
//! children after creation.

use mimalloc::MiMalloc;
use numpy::{Complex64, PyArray1};
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError, PyValueError},
    prelude::*,
};
use schedule::ElementCommonBuilder;
use std::{collections::HashMap, sync::Arc};

use crate::sampler::Sampler;

mod sampler;
mod schedule;
mod shape;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Channel configuration.
#[pyclass(get_all, frozen)]
#[derive(Debug, Clone)]
struct Channel {
    name: String,
    base_freq: f64,
    sample_rate: f64,
    length: usize,
    delay: f64,
    align_level: i32,
}

#[pymethods]
impl Channel {
    #[new]
    #[pyo3(signature = (name, base_freq, sample_rate, length, *, delay=0.0, align_level=-10))]
    fn new(
        name: String,
        base_freq: f64,
        sample_rate: f64,
        length: usize,
        delay: f64,
        align_level: i32,
    ) -> Self {
        Channel {
            name,
            base_freq,
            sample_rate,
            length,
            delay,
            align_level,
        }
    }
}

/// Alignment of a schedule element.
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
            return Ok(shape::Shape::new_interp(
                interp.knots.clone(),
                interp.controls.clone(),
                interp.degree,
            ));
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
    let msg = "Failed to convert the value to (f64, f64).";
    Err(PyValueError::new_err(msg))
}

/// Base class for schedule elements.
///
/// A schedule element is a node in the tree structure of a schedule similar to
/// HTML elements. The design is inspired by `XAML in WPF / WinUI
/// <https://learn.microsoft.com/en-us/windows/apps/design/layout/layouts-with-xaml>`_
///
/// When :attr:`duration`, :attr:`max_duration`, and :attr:`min_duration` are
/// conflicting, the priority is as follows:
///
/// 1. :attr:`min_duration`
/// 2. :attr:`max_duration`
/// 3. :attr:`duration`
#[pyclass(subclass, frozen)]
#[derive(Debug, Clone)]
struct Element(Arc<schedule::Element>);

#[pymethods]
impl Element {
    /// tuple[float, float]: Margin of the element.
    #[getter]
    fn margin(&self) -> (f64, f64) {
        self.0.common().margin()
    }

    /// Alignment: Alignment of the element.
    #[getter]
    fn alignment(&self) -> Alignment {
        self.0.common().alignment()
    }

    /// bool: Whether the element is a phantom element and should not add to waveforms.
    #[getter]
    fn phantom(&self) -> bool {
        self.0.common().phantom()
    }

    /// float | None: Duration of the element.
    #[getter]
    fn duration(&self) -> Option<f64> {
        self.0.common().duration()
    }

    /// float: Maximum duration of the element.
    #[getter]
    fn max_duration(&self) -> f64 {
        self.0.common().max_duration()
    }

    /// float: Minimum duration of the element.
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
/// If :attr:`flexible` is set to ``True`` and :attr:`alignment` is set to
/// :attr:`Alignment.Stretch`, the plateau of the pulse is stretched to fill the
/// parent element.
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
        channel_id: usize,
        shape_id: Option<usize>,
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

    /// int: Target channel ID.
    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .channel_id();
        Ok(ret)
    }

    /// int | None: Shape ID of the pulse. If ``None``, the pulse is a
    ///     rectangular pulse.
    #[getter]
    fn shape_id(slf: &Bound<'_, Self>) -> PyResult<Option<usize>> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_play()
            .ok_or(PyValueError::new_err(
                "Failed to get the play variant from the element.",
            ))?
            .shape_id();
        Ok(ret)
    }

    /// float: Amplitude of the pulse.
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

    /// float: Width of the pulse.
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

    /// float: Plateau of the pulse.
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

    /// float: Drag coefficient of the pulse. If the pulse is a rectangular pulse,
    ///     the drag coefficient is ignored.
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

    /// float: Additional frequency of the pulse on top of channel base
    ///     frequency and frequency shift.
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

    /// float: Additional phase of the pulse in **cycles**.
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

    /// bool: Whether the pulse is flexible and should stretch to fill the parent.
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
        channel_id: usize,
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

    /// int: Target channel ID.
    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_shift_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the shift_phase variant from the element.",
            ))?
            .channel_id();
        Ok(ret)
    }

    /// float: Phase shift in **cycles**.
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
/// Given the base frequency :math:`f`, the frequency shift :math:`\\Delta f`,
/// the time :math:`t`, and the phase offset :math:`\\phi_0`, the phase is
/// defined as
///
/// .. math::
///
///     \\phi(t) = (f + \\Delta f) t + \\phi_0
///
/// :class:`SetPhase` sets the phase offset :math:`\\phi_0` such that
/// :math:`\\phi(t)` is equal to the given phase.
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
        channel_id: usize,
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

    /// int: Target channel ID.
    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_set_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the set_phase variant from the element.",
            ))?
            .channel_id();
        Ok(ret)
    }

    /// float: Target phase value in **cycles**.
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
/// Additional frequency shift on top of the channel cumulative frequency shift.
/// Phase offset will be adjusted accordingly such that the phase is continuous
/// at the shift point.
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
        channel_id: usize,
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

    /// int: Target channel ID.
    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_shift_freq()
            .ok_or(PyValueError::new_err(
                "Failed to get the shift_freq variant from the element.",
            ))?
            .channel_id();
        Ok(ret)
    }

    /// float: Delta frequency.
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
/// Set the channel frequency shift to the target frequency. Phase offset will
/// be adjusted accordingly such that the phase is continuous at the shift point.
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
        channel_id: usize,
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

    /// int: Target channel ID.
    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_set_freq()
            .ok_or(PyValueError::new_err(
                "Failed to get the set_freq variant from the element.",
            ))?
            .channel_id();
        Ok(ret)
    }

    /// float: Target frequency.
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
/// This instruction swaps carrier phases between two target channels at the
/// scheduled time point. Carrier phase is defined as
///
/// .. math::
///     \\phi(t) = (f + \\Delta f) t + \\phi_0
///
/// where :math:`f` is the base frequency, :math:`\\Delta f` is the frequency
/// shift, :math:`t` is the time, and :math:`\\phi_0` is the phase offset.
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
        channel_id1: usize,
        channel_id2: usize,
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

    /// int: Target channel ID 1.
    #[getter]
    fn channel_id1(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_swap_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the swap_phase variant from the element.",
            ))?
            .channel_id1();
        Ok(ret)
    }

    /// int: Target channel ID 2.
    #[getter]
    fn channel_id2(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let ret = slf
            .downcast::<Element>()?
            .get()
            .0
            .try_get_swap_phase()
            .ok_or(PyValueError::new_err(
                "Failed to get the swap_phase variant from the element.",
            ))?
            .channel_id2();
        Ok(ret)
    }
}

/// A barrier element.
///
/// A barrier element is a zero-duration no-op element. Useful for aligning
/// elements on different channels in :class:`Stack`.
///
/// If :attr:`channel_ids` is empty, the barrier is applied to
/// all channels in its parent element.
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
        channel_ids: Vec<usize>,
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

    /// Sequence[int]: Target channel IDs. The returned value is a copy of the
    ///     internal channel IDs.
    #[getter]
    fn channel_ids(slf: &Bound<'_, Self>) -> PyResult<Vec<usize>> {
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

/// A repeated schedule element.
#[pyclass(extends=Element, get_all, frozen)]
#[derive(Debug, Clone)]
struct Repeat {
    /// Element: Child element to repeat.
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

    /// int: Number of repetitions.
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

    /// float: Spacing between repetitions.
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

/// Direction of arrangement.
#[pyclass(frozen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Backward,
    Forward,
}

#[pymethods]
impl Direction {
    /// Convert the value to Direction.
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

/// Layout child elements in one direction.
///
/// The child elements are arranged in one direction. The direction can be
/// forwards or backwards.
///
/// Child elements with no common channel are arranged in parallel.
/// :class:`Barrier` can be used to synchronize multiple channels.
#[pyclass(extends=Element, get_all, frozen)]
#[derive(Debug, Clone)]
struct Stack {
    /// Sequence[Element]: Child elements.
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

    /// Direction: Direction of arrangement.
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

/// A child element with an absolute time.
#[pyclass(get_all, frozen)]
#[derive(Debug, Clone)]
struct AbsoluteEntry {
    /// float: Time relative to the start of the parent element.
    time: f64,
    /// Element: Child element.
    element: Py<Element>,
}

#[pymethods]
impl AbsoluteEntry {
    #[new]
    fn new(time: f64, element: Py<Element>) -> Self {
        AbsoluteEntry { time, element }
    }

    /// Convert the value to AbsoluteEntry.
    ///
    /// the value can be:
    /// - AbsoluteEntry
    /// - Element
    /// - tuple[float, Element]
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(element) = obj.extract() {
            return Py::new(py, AbsoluteEntry::new(0.0, element));
        }
        if let Ok((time, element)) = obj.extract() {
            return Py::new(py, AbsoluteEntry::new(time, element));
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to AbsoluteEntry",
        ))
    }
}

fn extract_absolute_entry(obj: &Bound<'_, PyAny>) -> PyResult<AbsoluteEntry> {
    AbsoluteEntry::convert(obj).and_then(|x| x.extract(obj.py()))
}

/// An absolute schedule element.
///
/// The child elements are arranged in absolute time. The time of each child
/// element is relative to the start of the absolute schedule. The duration of
/// the absolute schedule is the maximum end time of the child elements.
#[pyclass(extends=Element, get_all, frozen)]
#[derive(Debug, Clone)]
struct Absolute {
    /// Sequence[AbsoluteEntry]: Child elements.
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
/// - Seconds: Fixed length in seconds.
/// - Auto: Auto length.
/// - Star: Ratio of the remaining space.
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
/// length can be specified in seconds, as a fraction of the remaining space,
/// or automatically.
#[pyclass(get_all, frozen)]
#[derive(Debug, Clone)]
struct GridLength {
    value: f64,
    unit: GridLengthUnit,
}

#[pymethods]
impl GridLength {
    #[new]
    fn new(value: f64, unit: GridLengthUnit) -> Self {
        GridLength { value, unit }
    }

    /// Create an automatic grid length.
    #[staticmethod]
    fn auto() -> Self {
        GridLength {
            value: 0.0,
            unit: GridLengthUnit::Auto,
        }
    }

    /// Create a ratio based grid length.
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
    /// - GridLength
    /// - float: Fixed length in seconds.
    /// - 'auto': Auto length.
    /// - 'x*': x stars.
    /// - 'x': Fixed length in seconds.
    /// - '*': 1 star.
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

/// A child element in a grid.
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
    fn new(element: Py<Element>, column: usize, span: usize) -> Self {
        GridEntry {
            element,
            column,
            span,
        }
    }

    /// Convert the value to GridEntry.
    ///
    /// The value can be:
    /// - GridEntry
    /// - Element
    /// - tuple[Element, int]: Element and column.
    /// - tuple[Element, int, int]: Element, column, and span.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(element) = obj.extract() {
            return Py::new(py, GridEntry::new(element, 0, 1));
        }
        if let Ok((element, column)) = obj.extract() {
            return Py::new(py, GridEntry::new(element, column, 1));
        }
        if let Ok((element, column, span)) = obj.extract() {
            return Py::new(py, GridEntry::new(element, column, span));
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to GridEntry.",
        ))
    }
}

fn extract_grid_entry(obj: &Bound<'_, PyAny>) -> PyResult<GridEntry> {
    GridEntry::convert(obj).and_then(|x| x.extract(obj.py()))
}

/// A grid schedule element.
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

    /// Sequence[GridLength]: Column lengths.
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
///     channels (Iterable[Channel]): Information of the channels.
///     shapes (Iterable[Shape]): Shapes used in the schedule.
///     schedule (Element): Schedule to execute.
///     time_tolerance (float): Tolerance for time comparison. Default is 1e-12.
///     amp_tolerance (float): Tolerance for amplitude comparison. Default is 0.1 / 2^16.
///     phase_tolerance (float): Tolerance for phase comparison. Default is 1e-4.
///     allow_oversize (bool): Allow oversize elements. Default is ``False``.
///
/// Returns:
///     Dict[str, numpy.ndarray]: Waveforms of the channels.
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
///         channels = [Channel("xy", 30e6, 2e9, 1000)]
///         shapes = [Hann()]
///         schedule = Stack(duration=500e-9).with_children(
///             Play(
///                 channel_id=0,
///                 shape_id=0,
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
    phase_tolerance=1e-4,
    allow_oversize=false,
))]
#[allow(clippy::too_many_arguments)]
fn generate_waveforms(
    py: Python<'_>,
    channels: Vec<Channel>,
    shapes: Vec<Py<Shape>>,
    schedule: &Bound<'_, Element>,
    time_tolerance: f64,
    amp_tolerance: f64,
    phase_tolerance: f64,
    allow_oversize: bool,
) -> PyResult<HashMap<String, Py<PyArray1<Complex64>>>> {
    // TODO: use the tolerances
    let _ = (amp_tolerance, phase_tolerance);
    let root = schedule.downcast::<Element>()?.get().0.clone();
    let measured = schedule::measure(root, f64::INFINITY);
    let arrange_options = schedule::ScheduleOptions {
        time_tolerance,
        allow_oversize,
    };
    let arranged = schedule::arrange(&measured, 0.0, measured.duration(), &arrange_options)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let mut sampler = Sampler::new();
    for c in channels.iter() {
        sampler.add_channel(c.base_freq, c.sample_rate, c.length, c.delay);
    }
    for s in shapes.iter() {
        let s = s.bind(py);
        sampler.add_shape(Shape::get_rust_shape(s)?);
    }
    sampler.execute(&arranged);
    let results = sampler.into_result();
    let dict = channels
        .into_iter()
        .zip(results)
        .map(|(c, w)| (c.name, PyArray1::from_vec_bound(py, w).unbind()))
        .collect();
    Ok(dict)
}

/// Generates microwave pulses for superconducting quantum computing experiments.
///
/// .. caution::
///
///     All phase values are in number of cycles. For example, a phase of
///     :math:`0.25` means :math:`\\pi/2` radians.
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
