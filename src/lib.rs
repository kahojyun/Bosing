//! Although Element struct may contains [`Py<Element>`] as children, it is not
//! possible to create cyclic references because we don't allow mutate the
//! children after creation.

use pyo3::{prelude::*, types::PyTuple};


/// Biquad filter
#[pyclass(get_all)]
#[derive(Clone, Debug)]
pub struct Biquad {
    /// float: b0
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
}

#[pymethods]
impl Biquad {
    #[new]
    fn new(b0: f64, b1: f64, b2: f64, a1: f64, a2: f64) -> Self {
        Biquad { b0, b1, b2, a1, a2 }
    }

    fn __repr__(&self) -> String {
        format!(
            "Biquad(b0={}, b1={}, b2={}, a1={}, a2={})",
            self.b0, self.b1, self.b2, self.a1, self.a2
        )
    }

    /// say_hello() -> str
    /// 
    /// Say hello from Rust
    pub fn say_hello(&self) -> PyResult<String> {
        Ok("Hello from Rust!".to_string())
    }

    /// get_array() -> numpy.ndarray
    /// 
    /// Return the coefficients as a numpy array.
    pub fn get_array(&self) -> PyResult<Vec<f64>> {
        Ok(vec![self.b0, self.b1, self.b2, self.a1, self.a2])
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct IqCalibration {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    i_offset: f64,
    q_offset: f64,
}

#[pymethods]
impl IqCalibration {
    #[new]
    fn new(a: f64, b: f64, c: f64, d: f64, i_offset: Option<f64>, q_offset: Option<f64>) -> Self {
        let i_offset = i_offset.unwrap_or(0.0);
        let q_offset = q_offset.unwrap_or(0.0);
        IqCalibration {
            a,
            b,
            c,
            d,
            i_offset,
            q_offset,
        }
    }
}

#[pyclass(get_all)]
#[derive(Clone, Debug)]
struct Channel {
    name: String,
    base_freq: f64,
    sample_rate: f64,
    length: usize,
    delay: f64,
    align_level: i32,
    iq_calibration: Option<IqCalibration>,
    iir: Vec<Biquad>,
    fir: Vec<f64>,
}

#[pymethods]
impl Channel {
    #[new]
    fn new(
        name: String,
        base_freq: f64,
        sample_rate: f64,
        length: usize,
        delay: Option<f64>,
        align_level: Option<i32>,
        iq_calibration: Option<IqCalibration>,
        iir: Option<Vec<Biquad>>,
        fir: Option<Vec<f64>>,
    ) -> Self {
        let delay = delay.unwrap_or(0.0);
        let align_level = align_level.unwrap_or(-10);
        let iir = iir.unwrap_or_default();
        let fir = fir.unwrap_or_default();
        Channel {
            name,
            base_freq,
            sample_rate,
            length,
            delay,
            align_level,
            iq_calibration,
            iir,
            fir,
        }
    }
}

#[pyclass]
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
    fn new(
        time_tolerance: Option<f64>,
        amp_tolerance: Option<f64>,
        phase_tolerance: Option<f64>,
        allow_oversize: Option<bool>,
    ) -> Self {
        let time_tolerance = time_tolerance.unwrap_or(1e-12);
        let amp_tolerance = amp_tolerance.unwrap_or(0.1 / 2f64.powi(16));
        let phase_tolerance = phase_tolerance.unwrap_or(1e-4);
        let allow_oversize = allow_oversize.unwrap_or(false);
        Options {
            time_tolerance,
            amp_tolerance,
            phase_tolerance,
            allow_oversize,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            time_tolerance: 1e-12,
            amp_tolerance: 0.1 / 2f64.powi(16),
            phase_tolerance: 1e-4,
            allow_oversize: false,
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

#[pyclass(subclass)]
#[derive(Clone, Debug)]
struct ShapeInfo;

#[pyclass(extends=ShapeInfo)]
#[derive(Clone, Debug)]
struct Hann;

#[pymethods]
impl Hann {
    #[new]
    fn new() -> (Self, ShapeInfo) {
        (Self, ShapeInfo)
    }
}

#[pyclass(extends=ShapeInfo)]
#[derive(Clone, Debug)]
struct Interp {
    x_array: Vec<f64>,
    y_array: Vec<f64>,
}

#[pymethods]
impl Interp {
    #[new]
    fn new(x_array: Vec<f64>, y_array: Vec<f64>) -> (Self, ShapeInfo) {
        (Self { x_array, y_array }, ShapeInfo)
    }
}

#[pyclass(subclass)]
#[derive(Clone, Debug)]
pub struct Element {
    margin: (f64, f64),
    alignment: Alignment,
    visibility: bool,
    duration: Option<f64>,
    max_duration: f64,
    min_duration: f64,
}

impl Element {
    fn new(
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> Self {
        let margin = margin.unwrap_or((0.0, 0.0));
        let alignment = alignment.unwrap_or(Alignment::End);
        let visibility = visibility.unwrap_or(true);
        let max_duration = max_duration.unwrap_or(f64::INFINITY);
        let min_duration = min_duration.unwrap_or(0.0);
        Element {
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        }
    }
}

#[pyclass(extends=Element)]
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
    fn new(
        channel_id: usize,
        amplitude: f64,
        shape_id: usize,
        width: f64,
        plateau: Option<f64>,
        drag_coef: Option<f64>,
        frequency: Option<f64>,
        phase: Option<f64>,
        flexible: Option<bool>,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> (Self, Element) {
        let plateau = plateau.unwrap_or(0.0);
        let drag_coef = drag_coef.unwrap_or(0.0);
        let frequency = frequency.unwrap_or(0.0);
        let phase = phase.unwrap_or(0.0);
        let flexible = flexible.unwrap_or(false);
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        (
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
        )
    }
}

#[pyclass(extends=Element)]
#[derive(Clone, Debug)]
struct ShiftPhase {
    channel_id: usize,
    phase: f64,
}

#[pymethods]
impl ShiftPhase {
    #[new]
    fn new(
        channel_id: usize,
        phase: f64,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> (Self, Element) {
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        (Self { channel_id, phase }, element)
    }
}

#[pyclass(extends=Element)]
#[derive(Clone, Debug)]
struct SetPhase {
    channel_id: usize,
    phase: f64,
}

#[pymethods]
impl SetPhase {
    #[new]
    fn new(
        channel_id: usize,
        phase: f64,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> (Self, Element) {
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        (Self { channel_id, phase }, element)
    }
}

#[pyclass(extends=Element)]
#[derive(Clone, Debug)]
struct ShiftFreq {
    channel_id: usize,
    frequency: f64,
}

#[pymethods]
impl ShiftFreq {
    #[new]
    fn new(
        channel_id: usize,
        frequency: f64,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> (Self, Element) {
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        (
            Self {
                channel_id,
                frequency,
            },
            element,
        )
    }
}

#[pyclass(extends=Element)]
#[derive(Clone, Debug)]
struct SetFreq {
    channel_id: usize,
    frequency: f64,
}

#[pymethods]
impl SetFreq {
    #[new]
    fn new(
        channel_id: usize,
        frequency: f64,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> (Self, Element) {
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        (
            Self {
                channel_id,
                frequency,
            },
            element,
        )
    }
}

#[pyclass(extends=Element)]
#[derive(Clone, Debug)]
struct SwapPhase {
    channel_id1: usize,
    channel_id2: usize,
}

#[pymethods]
impl SwapPhase {
    #[new]
    fn new(
        channel_id1: usize,
        channel_id2: usize,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> (Self, Element) {
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        (
            Self {
                channel_id1,
                channel_id2,
            },
            element,
        )
    }
}

#[pyclass(extends=Element)]
#[derive(Clone, Debug)]
struct Barrier {
    channel_ids: Vec<usize>,
}

#[pymethods]
impl Barrier {
    #[new]
    fn new(
        channel_ids: Option<Vec<usize>>,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> (Self, Element) {
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        let channel_ids = channel_ids.unwrap_or_default();
        (Self { channel_ids }, element)
    }
}

#[pyclass(extends=Element)]
#[derive(Clone, Debug)]
struct Repeat {
    child: Py<Element>,
    count: usize,
    spacing: f64,
}

#[pymethods]
impl Repeat {
    #[new]
    fn new(
        child: Py<Element>,
        count: usize,
        spacing: f64,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> (Self, Element) {
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        (
            Self {
                child,
                count,
                spacing,
            },
            element,
        )
    }
}

#[pyclass]
#[derive(Clone, Debug)]
enum ArrangeDirection {
    Backwards,
    Forwards,
}

impl ArrangeDirection {
    fn from_str(s: &str) -> Self {
        match s {
            "backwards" => ArrangeDirection::Backwards,
            "forwards" => ArrangeDirection::Forwards,
            _ => panic!("Invalid ArrangeDirection"),
        }
    }

    fn from_py(s: &Bound<PyAny>) -> PyResult<Self> {
        if let Ok(v) = s.extract() {
            Ok(v)
        } else {
            let s = s.extract::<&str>()?;
            Ok(ArrangeDirection::from_str(s))
        }
    }
}

#[pyclass(extends=Element)]
#[derive(Clone, Debug)]
struct Stack {
    children: Vec<Py<Element>>,
    direction: ArrangeDirection,
}

#[pymethods]
impl Stack {
    #[new]
    fn new(
        children: Option<Vec<Py<Element>>>,
        direction: Option<&Bound<PyAny>>,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> PyResult<(Self, Element)> {
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        let children = children.unwrap_or_default();
        let direction = direction
            .map(ArrangeDirection::from_py)
            .unwrap_or(Ok(ArrangeDirection::Backwards))?;
        Ok((
            Self {
                children,
                direction,
            },
            element,
        ))
    }

    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<Self>, children: &Bound<PyTuple>) -> PyResult<Py<Self>> {
        let children = children.extract()?;
        let stack = Self {
            children,
            direction: slf.borrow().direction.clone(),
        };
        let base = slf.downcast::<Element>()?.borrow().clone();
        let py = slf.py();
        Py::new(py, (stack, base))
    }
}

#[pyclass]
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
}

#[pyclass(extends=Element)]
#[derive(Clone, Debug)]
struct Absolute {
    children: Vec<AbsoluteEntry>,
}

#[pymethods]
impl Absolute {
    #[new]
    fn new(
        children: Option<Vec<AbsoluteEntry>>,
        margin: Option<(f64, f64)>,
        alignment: Option<Alignment>,
        visibility: Option<bool>,
        duration: Option<f64>,
        max_duration: Option<f64>,
        min_duration: Option<f64>,
    ) -> (Self, Element) {
        let element = Element::new(
            margin,
            alignment,
            visibility,
            duration,
            max_duration,
            min_duration,
        );
        let children = children.unwrap_or_default();
        (Self { children }, element)
    }

    #[pyo3(signature=(*children))]
    fn with_children(slf: &Bound<Self>, children: &Bound<PyTuple>) -> PyResult<Py<Self>> {
        let children = children.extract()?;
        let stack = Self { children };
        let base = slf.downcast::<Element>()?.borrow().clone();
        let py = slf.py();
        Py::new(py, (stack, base))
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct Request {
    channels: Vec<Channel>,
    shapes: Vec<ShapeInfo>,
    schedule: Py<Element>,
    options: Options,
}

#[pymethods]
impl Request {
    #[new]
    fn new(
        channels: Vec<Channel>,
        shapes: Vec<ShapeInfo>,
        schedule: Py<Element>,
        options: Option<Options>,
    ) -> Self {
        let options = options.unwrap_or_default();
        Request {
            channels,
            shapes,
            schedule,
            options,
        }
    }
}

/// add(a: int, b: int) -> int
/// 
/// A simple function that adds two numbers together.
#[pyfunction]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// A Python module implemented in Rust.
#[pymodule]
fn bosing_rs(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Biquad>()?;
    m.add_class::<IqCalibration>()?;
    m.add_class::<Channel>()?;
    m.add_class::<Options>()?;
    m.add_class::<Hann>()?;
    m.add_class::<Interp>()?;
    m.add_class::<Play>()?;
    m.add_class::<ShiftPhase>()?;
    m.add_class::<SetPhase>()?;
    m.add_class::<ShiftFreq>()?;
    m.add_class::<SetFreq>()?;
    m.add_class::<SwapPhase>()?;
    m.add_class::<Barrier>()?;
    m.add_class::<Repeat>()?;
    m.add_class::<Stack>()?;
    m.add_class::<AbsoluteEntry>()?;
    m.add_class::<Absolute>()?;
    m.add_class::<Request>()?;
    m.add_class::<ShapeInfo>()?;
    m.add_class::<Element>()?;
    m.add_function(wrap_pyfunction!(add, m)?)?;
    Ok(())
}
