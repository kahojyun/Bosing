//! Although Element struct may contains [`Py<Element>`] as children, it is not
//! possible to create cyclic references because we don't allow mutate the
//! children after creation.

use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use schedule::ElementCommonBuilder;
use std::{sync::Arc, time::Instant};

mod schedule;
mod shape;

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

#[pyclass(get_all, frozen)]
#[derive(Debug, Clone)]
struct Options {
    time_tolerance: f64,
    amp_tolerance: f64,
    phase_tolerance: f64,
    allow_oversize: bool,
}

#[pymethods]
impl Options {
    #[new]
    #[pyo3(signature = (
        *,
        time_tolerance=1e-12,
        amp_tolerance=0.1 / 2f64.powi(16),
        phase_tolerance=1e-4,
        allow_oversize=false,
    ))]
    fn new(
        time_tolerance: f64,
        amp_tolerance: f64,
        phase_tolerance: f64,
        allow_oversize: bool,
    ) -> Self {
        Options {
            time_tolerance,
            amp_tolerance,
            phase_tolerance,
            allow_oversize,
        }
    }
}

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

#[pyclass(subclass, frozen)]
#[derive(Debug, Clone)]
struct Shape;

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
        let variant = schedule::Barrier::from_channel_ids(channel_ids);
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

#[pyclass(frozen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Backward,
    Forward,
}

#[pymethods]
impl Direction {
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

#[pyclass(get_all, frozen)]
#[derive(Debug, Clone)]
struct AbsoluteEntry {
    time: f64,
    element: Py<Element>,
}

#[pymethods]
impl AbsoluteEntry {
    #[new]
    fn new(time: f64, element: Py<Element>) -> Self {
        AbsoluteEntry { time, element }
    }

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

#[pyclass(frozen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridLengthUnit {
    Seconds,
    Auto,
    Star,
}

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

    #[staticmethod]
    fn auto() -> Self {
        GridLength {
            value: 0.0,
            unit: GridLengthUnit::Auto,
        }
    }

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
    shapes: Vec<Shape>,
    schedule: &Bound<'_, Element>,
    time_tolerance: f64,
    amp_tolerance: f64,
    phase_tolerance: f64,
    allow_oversize: bool,
) -> PyResult<Py<PyDict>> {
    let t0 = Instant::now();
    let root = schedule.downcast::<Element>()?.get().0.clone();
    let measured = schedule::measure(root, f64::INFINITY);
    let arrange_options = schedule::ScheduleOptions {
        time_tolerance,
        allow_oversize,
    };
    let _ = schedule::arrange(&measured, 0.0, measured.duration(), &arrange_options);
    let t1 = Instant::now();
    println!("Arrangement time: {:?}", t1 - t0);
    Ok(PyDict::new_bound(py).into())
}

/// A Python module implemented in Rust.
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
    m.add_class::<Options>()?;
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
