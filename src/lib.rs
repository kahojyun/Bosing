//! Although Element struct may contains [`Py<Element>`] as children, it is not
//! possible to create cyclic references because we don't allow mutate the
//! children after creation.

mod schedule;
mod shape;

use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};

#[pyclass(get_all)]
#[derive(Clone, Debug)]
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

#[pyclass(get_all)]
#[derive(Clone, Debug)]
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

#[pyclass]
#[derive(Clone, Debug)]
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

#[pyclass(subclass)]
#[derive(Clone, Debug)]
struct Shape;

#[pyclass(extends=Shape)]
#[derive(Clone, Debug)]
struct Hann;

#[pymethods]
impl Hann {
    #[new]
    fn new() -> (Self, Shape) {
        (Self, Shape)
    }
}

#[pyclass(extends=Shape, get_all)]
#[derive(Clone, Debug)]
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

#[pyclass(subclass, get_all)]
#[derive(Clone, Debug)]
struct Element {
    margin: (f64, f64),
    alignment: Alignment,
    phantom: bool,
    duration: Option<f64>,
    max_duration: f64,
    min_duration: f64,
}

impl Element {
    fn new(
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<f64>,
        max_duration: f64,
        min_duration: f64,
    ) -> PyResult<Self> {
        let margin = match margin {
            Some(margin) => extract_margin(margin)?,
            None => (0.0, 0.0),
        };
        let alignment = match alignment {
            Some(alignment) => extract_alignment(alignment)?,
            None => Alignment::End,
        };
        Ok(Element {
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        })
    }
}

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct Play {
    channel_id: usize,
    amplitude: f64,
    shape_id: usize,
    width: f64,
    plateau: f64,
    drag_coef: f64,
    frequency: f64,
    phase: f64,
    flexible: bool,
}

#[pymethods]
impl Play {
    #[new]
    #[pyo3(signature = (
        channel_id,
        amplitude,
        shape_id,
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
        amplitude: f64,
        shape_id: usize,
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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        Ok((
            Self {
                channel_id,
                amplitude,
                shape_id,
                width,
                plateau,
                drag_coef,
                frequency,
                phase,
                flexible,
            },
            element,
        ))
    }
}

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct ShiftPhase {
    channel_id: usize,
    phase: f64,
}

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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        Ok((Self { channel_id, phase }, element))
    }
}

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct SetPhase {
    channel_id: usize,
    phase: f64,
}

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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        Ok((Self { channel_id, phase }, element))
    }
}

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct ShiftFreq {
    channel_id: usize,
    frequency: f64,
}

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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        Ok((
            Self {
                channel_id,
                frequency,
            },
            element,
        ))
    }
}

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct SetFreq {
    channel_id: usize,
    frequency: f64,
}

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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        Ok((
            Self {
                channel_id,
                frequency,
            },
            element,
        ))
    }
}

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct SwapPhase {
    channel_id1: usize,
    channel_id2: usize,
}

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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        Ok((
            Self {
                channel_id1,
                channel_id2,
            },
            element,
        ))
    }
}

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct Barrier {
    channel_ids: Vec<usize>,
}

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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        Ok((Self { channel_ids }, element))
    }
}

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct Repeat {
    child: Py<Element>,
    count: usize,
    spacing: f64,
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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        Ok((
            Self {
                child,
                count,
                spacing,
            },
            element,
        ))
    }
}

#[pyclass]
#[derive(Clone, Debug)]
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

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct Stack {
    children: Vec<Py<Element>>,
    direction: Direction,
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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        let direction = match direction {
            Some(direction) => extract_direction(direction)?,
            None => Direction::Backward,
        };
        Ok((
            Self {
                children,
                direction,
            },
            element,
        ))
    }

    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<'_, Self>, children: Vec<Py<Element>>) -> PyResult<Py<Self>> {
        let new = Self {
            children,
            direction: slf.borrow().direction.clone(),
        };
        let base = slf.downcast::<Element>()?.borrow().clone();
        let py = slf.py();
        Py::new(py, (new, base))
    }
}

#[pyclass(get_all)]
#[derive(Clone, Debug)]
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

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        let children = children
            .into_iter()
            .map(|x| extract_absolute_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        Ok((Self { children }, element))
    }

    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<'_, Self>, children: Vec<Py<PyAny>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let children = children
            .into_iter()
            .map(|x| extract_absolute_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let new = Self { children };
        let base = slf.downcast::<Element>()?.borrow().clone();
        Py::new(py, (new, base))
    }
}

#[pyclass]
#[derive(Clone, Debug)]
enum GridLengthUnit {
    Seconds,
    Auto,
    Star,
}

#[pyclass(get_all)]
#[derive(Clone, Debug)]
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
    fn star(value: f64) -> Self {
        GridLength {
            value,
            unit: GridLengthUnit::Star,
        }
    }

    #[staticmethod]
    fn fixed(value: f64) -> Self {
        GridLength {
            value,
            unit: GridLengthUnit::Seconds,
        }
    }

    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(v) = obj.extract() {
            return Py::new(py, GridLength::fixed(v));
        }
        if let Ok(s) = obj.extract::<&str>() {
            if s == "auto" {
                return Py::new(py, GridLength::auto());
            }
            if let Some(v) = s.strip_suffix('*').and_then(|x| x.parse().ok()) {
                return Py::new(py, GridLength::star(v));
            }
            if let Ok(v) = s.parse() {
                return Py::new(py, GridLength::fixed(v));
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

#[pyclass(get_all)]
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

#[pyclass(extends=Element, get_all)]
#[derive(Clone, Debug)]
struct Grid {
    children: Vec<GridEntry>,
    columns: Vec<GridLength>,
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
        let element = Element::new(
            margin,
            alignment,
            phantom,
            duration,
            max_duration,
            min_duration,
        )?;
        let children = children
            .into_iter()
            .map(|x| extract_grid_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let columns = columns
            .into_iter()
            .map(|x| extract_grid_length(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        Ok((Self { children, columns }, element))
    }

    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<'_, Self>, children: Vec<Py<PyAny>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let children = children
            .into_iter()
            .map(|x| extract_grid_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let new = Self {
            children,
            columns: slf.borrow().columns.clone(),
        };
        let base = slf.downcast::<Element>()?.borrow().clone();
        Py::new(py, (new, base))
    }
}

#[pyfunction]
fn generate_waveforms(
    channels: Vec<Channel>,
    shapes: Vec<Shape>,
    schedule: &Bound<'_, Element>,
    options: Option<Options>,
) -> PyResult<Py<PyDict>> {
    let _ = channels;
    let _ = shapes;
    let _ = schedule;
    let _ = options;
    todo!()
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
