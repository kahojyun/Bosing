//! Although Element struct may contains [`Py<Element>`] as children, it is not
//! possible to create cyclic references because we don't allow mutate the
//! children after creation.
mod executor;
mod plot;
mod pulse;
mod python;
mod quant;
mod schedule;
mod shape;
mod util;

use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    str::FromStr,
    sync::Arc,
};

use hashbrown::HashMap;
use itertools::Itertools;
use ndarray::ArrayViewMut2;
use numpy::{prelude::*, AllowTypeChange, PyArray2, PyArrayLike2};
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError, PyValueError},
    intern,
    prelude::*,
    types::{DerefToPyAny, PyList},
};
use rayon::prelude::*;

use crate::{
    executor::Executor,
    pulse::{
        apply_fir_inplace, apply_iir_inplace, apply_iq_inplace, apply_offset_inplace, PulseList,
        Sampler,
    },
    python::{Arg, FirArray, IirArray, IqMatrix, OffsetArray},
    quant::{Amplitude, ChannelId, Frequency, Phase, ShapeId, Time},
    schedule::{ElementCommon, ElementCommonBuilder, ElementRef, ElementVariant, Measure},
};

#[derive(Debug, Clone, FromPyObject)]
struct Channel {
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

macro_rules! repr_list {
    ($self:ident, $py:ident, $($pos:ident),* $(,)*; $($kw:ident),* $(,)*; $($kwn:ident=$kwd:expr),* $(,)*) => {
        [
            $(Arg::positional(&$self.$pos, $py),)*
            $(Arg::keyword(
                intern!($py, stringify!($kw)).clone().unbind(),
                &$self.$kw,
                $py,
            ),)*
            $(Arg::key_with_default(
                intern!($py, stringify!($kwn)).clone().unbind(),
                &$self.$kwn,
                &$kwd,
                $py,
            ),)*
        ]
    };
}

trait RichRepr {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg>;

    fn to_rich_repr(&self, py: Python) -> Vec<Arg> {
        self.repr(py).into_iter().collect()
    }

    fn to_repr(&self, cls_name: impl Display, py: Python) -> PyResult<String> {
        Ok(format!(
            "{}({})",
            cls_name,
            self.repr(py)
                .into_iter()
                .map(|x| x.fmt(py))
                .flatten_ok()
                .collect::<PyResult<Vec<_>>>()?
                .join(", ")
        ))
    }
}

#[pyclass(module = "bosing", get_all, set_all)]
#[derive(Debug, Clone, Copy)]
struct OscState {
    base_freq: Frequency,
    delta_freq: Frequency,
    phase: Phase,
}

#[pymethods]
impl OscState {
    #[new]
    fn new(base_freq: Frequency, delta_freq: Frequency, phase: Phase) -> Self {
        OscState {
            base_freq,
            delta_freq,
            phase,
        }
    }

    fn total_freq(&self) -> Frequency {
        executor::OscState::from(*self).total_freq()
    }

    fn phase_at(&self, time: Time) -> Phase {
        executor::OscState::from(*self).phase_at(time)
    }

    fn with_time_shift(&self, time: Time) -> Self {
        executor::OscState::from(*self).with_time_shift(time).into()
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
#[pyclass(module = "bosing", frozen, eq)]
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
    fn convert(obj: &Bound<PyAny>) -> PyResult<Py<Self>> {
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(s) = obj.extract::<String>() {
            let alignment = match s.as_str() {
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

fn extract_alignment(obj: &Bound<PyAny>) -> PyResult<Alignment> {
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
#[pyclass(module = "bosing", subclass, frozen)]
#[derive(Debug, Clone)]
struct Shape;

impl Shape {
    fn get_rust_shape(slf: &Bound<Shape>) -> PyResult<shape::Shape> {
        if slf.downcast::<Hann>().is_ok() {
            return Ok(shape::Shape::new_hann());
        }
        if let Ok(interp) = slf.downcast::<Interp>() {
            let interp = interp.get();
            return Ok(shape::Shape::new_interp(
                interp.knots.clone(),
                interp.controls.clone(),
                interp.degree,
            )?);
        }
        Err(PyTypeError::new_err("Invalid shape type."))
    }
}

/// A Hann shape.
#[pyclass(module="bosing",extends=Shape, frozen)]
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
#[pyclass(module="bosing",extends=Shape, get_all, frozen)]
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

fn extract_margin(obj: &Bound<PyAny>) -> PyResult<(Time, Time)> {
    if let Ok(v) = obj.extract() {
        let t = Time::new(v)?;
        return Ok((t, t));
    }
    if let Ok((v1, v2)) = obj.extract() {
        let t1 = Time::new(v1)?;
        let t2 = Time::new(v2)?;
        return Ok((t1, t2));
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
#[pyclass(module = "bosing", subclass, frozen)]
#[derive(Debug, Clone)]
struct Element(ElementRef);

#[pymethods]
impl Element {
    #[getter]
    fn margin(&self) -> (Time, Time) {
        self.0.common.margin()
    }

    #[getter]
    fn alignment(&self) -> Alignment {
        self.0.common.alignment()
    }

    #[getter]
    fn phantom(&self) -> bool {
        self.0.common.phantom()
    }

    #[getter]
    fn duration(&self) -> Option<Time> {
        self.0.common.duration()
    }

    #[getter]
    fn max_duration(&self) -> Time {
        self.0.common.max_duration()
    }

    #[getter]
    fn min_duration(&self) -> Time {
        self.0.common.min_duration()
    }

    /// Measure the minimum total duration required by the element.
    ///
    /// This value includes both inner `duration` and outer `margin` of the element.
    ///
    /// This value is a *minimum* total duration wanted by the element. If the element is a child
    /// of other element, the final total duration will be determined by `alignment` option and
    /// parent container type.
    fn measure(&self) -> Time {
        self.0.measure()
    }

    /// Plot arrange result with the element as root.
    ///
    /// Args:
    ///     ax (matplotlib.axes.Axes | None): Axes to plot. If ``None``, `matplotlib.pyplot.gca` is
    ///         used.
    ///     channels (Sequence[str] | None): Channels to plot. If ``None``, all channels are
    ///         plotted.
    ///     max_depth (int): Maximum depth to plot. Defaults to 5.
    ///
    /// Returns:
    ///     matplotlib.axes.Axes: Axes with the plot.
    #[pyo3(signature = (ax=None, *, channels=None, max_depth=5))]
    fn plot(
        &self,
        py: Python,
        ax: Option<PyObject>,
        channels: Option<Vec<ChannelId>>,
        max_depth: usize,
    ) -> PyResult<PyObject> {
        let m = py.import_bound(intern!(py, "bosing._plot"))?;
        let plot_items = Box::new(plot::arrange_to_plot(self.0.clone()));
        let blocks = PlotIter { inner: plot_items };
        let channels = match channels {
            Some(channels) => PyList::new_bound(py, channels),
            None => PyList::new_bound(py, self.0.channels()),
        };
        let result = m.call_method1(intern!(py, "plot"), (ax, blocks, channels, max_depth))?;
        Ok(result.into())
    }
}

#[pyclass(module = "_bosing")]
struct PlotIter {
    inner: Box<dyn Iterator<Item = PlotItem> + Send>,
}

#[pymethods]
impl PlotIter {
    fn __iter__(slf: Bound<Self>) -> Bound<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        slf.inner.next().map(|x| x.into_py(slf.py()))
    }
}

#[pyclass(module = "_bosing", frozen, get_all)]
#[derive(Debug)]
struct PlotItem {
    channels: Vec<ChannelId>,
    start: Time,
    span: Time,
    depth: usize,
    kind: ItemKind,
}

#[pyclass(module = "_bosing", frozen, eq, hash)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ItemKind {
    Play,
    ShiftPhase,
    SetPhase,
    ShiftFreq,
    SetFreq,
    SwapPhase,
    Barrier,
    Repeat,
    Stack,
    Absolute,
    Grid,
}

impl ItemKind {
    fn from_variant(variant: &ElementVariant) -> Self {
        match variant {
            ElementVariant::Play(_) => Self::Play,
            ElementVariant::ShiftPhase(_) => Self::ShiftPhase,
            ElementVariant::SetPhase(_) => Self::SetPhase,
            ElementVariant::ShiftFreq(_) => Self::ShiftFreq,
            ElementVariant::SetFreq(_) => Self::SetFreq,
            ElementVariant::SwapPhase(_) => Self::SwapPhase,
            ElementVariant::Barrier(_) => Self::Barrier,
            ElementVariant::Repeat(_) => Self::Repeat,
            ElementVariant::Stack(_) => Self::Stack,
            ElementVariant::Absolute(_) => Self::Absolute,
            ElementVariant::Grid(_) => Self::Grid,
        }
    }
}

trait ElementSubclass: Sized + DerefToPyAny
where
    for<'a> &'a Self::Variant: TryFrom<&'a schedule::ElementVariant>,
    for<'a> <&'a Self::Variant as TryFrom<&'a schedule::ElementVariant>>::Error: Debug,
{
    type Variant: Into<schedule::ElementVariant>;

    fn inner<'a>(slf: &'a Bound<Self>) -> &'a ElementRef {
        slf.downcast::<Element>()
            .expect("Self should be a subclass of Element")
            .get()
            .0
            .borrow()
    }

    fn common<'a>(slf: &'a Bound<Self>) -> &'a ElementCommon {
        Self::inner(slf).common.borrow()
    }

    fn variant<'a>(slf: &'a Bound<Self>) -> &'a Self::Variant {
        Self::inner(slf)
            .variant
            .borrow()
            .try_into()
            .expect("Element should have a valid variant")
    }

    fn build_element(
        variant: Self::Variant,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
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
        let common = builder.build()?;
        Ok(Element(Arc::new(schedule::Element::new(common, variant))))
    }

    fn common_repr(slf: &Bound<Self>) -> impl IntoIterator<Item = Arg> {
        let py = slf.py();
        [
            Arg::key_with_default(
                intern!(py, "margin").clone().unbind(),
                Self::common(slf).margin(),
                (Time::ZERO, Time::ZERO),
                py,
            ),
            Arg::key_with_default(
                intern!(py, "alignment").clone().unbind(),
                Self::common(slf).alignment().into_py(py),
                Alignment::End.into_py(py),
                py,
            ),
            Arg::key_with_default(
                intern!(py, "phantom").clone().unbind(),
                Self::common(slf).phantom(),
                false,
                py,
            ),
            Arg::key_with_default(
                intern!(py, "duration").clone().unbind(),
                Self::common(slf).duration(),
                None,
                py,
            ),
            Arg::key_with_default(
                intern!(py, "max_duration").clone().unbind(),
                Self::common(slf).max_duration(),
                Time::INFINITY,
                py,
            ),
            Arg::key_with_default(
                intern!(py, "min_duration").clone().unbind(),
                Self::common(slf).min_duration(),
                Time::ZERO,
                py,
            ),
        ]
    }
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
#[pyclass(module="bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
struct Play;

impl ElementSubclass for Play {
    type Variant = schedule::Play;
}

#[pymethods]
impl Play {
    #[new]
    #[pyo3(signature = (
        channel_id,
        shape_id,
        amplitude,
        width,
        *,
        plateau=Time::ZERO,
        drag_coef=0.0,
        frequency=Frequency::ZERO,
        phase=Phase::ZERO,
        flexible=false,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: ChannelId,
        shape_id: Option<ShapeId>,
        amplitude: Amplitude,
        width: Time,
        plateau: Time,
        drag_coef: f64,
        frequency: Frequency,
        phase: Phase,
        flexible: bool,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::Play::new(channel_id, shape_id, amplitude, width)?
            .with_plateau(plateau)?
            .with_drag_coef(drag_coef)?
            .with_frequency(frequency)?
            .with_phase(phase)?
            .with_flexible(flexible);
        Ok((
            Self,
            Self::build_element(
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
    fn channel_id<'a>(slf: &'a Bound<Self>) -> &'a ChannelId {
        Self::variant(slf).channel_id()
    }

    #[getter]
    fn shape_id<'a>(slf: &'a Bound<Self>) -> Option<&'a ShapeId> {
        Self::variant(slf).shape_id()
    }

    #[getter]
    fn amplitude(slf: &Bound<Self>) -> Amplitude {
        Self::variant(slf).amplitude()
    }

    #[getter]
    fn width(slf: &Bound<Self>) -> Time {
        Self::variant(slf).width()
    }

    #[getter]
    fn plateau(slf: &Bound<Self>) -> Time {
        Self::variant(slf).plateau()
    }

    #[getter]
    fn drag_coef(slf: &Bound<Self>) -> f64 {
        Self::variant(slf).drag_coef()
    }

    #[getter]
    fn frequency(slf: &Bound<Self>) -> Frequency {
        Self::variant(slf).frequency()
    }

    #[getter]
    fn phase(slf: &Bound<Self>) -> Phase {
        Self::variant(slf).phase()
    }

    #[getter]
    fn flexible(slf: &Bound<Self>) -> bool {
        Self::variant(slf).flexible()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, Play> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        [
            Arg::positional(Play::variant(self).channel_id(), py),
            Arg::positional(Play::variant(self).shape_id(), py),
            Arg::positional(Play::variant(self).amplitude(), py),
            Arg::positional(Play::variant(self).width(), py),
            Arg::key_with_default(
                intern!(py, "plateau").clone().unbind(),
                Play::variant(self).plateau(),
                Time::ZERO,
                py,
            ),
            Arg::key_with_default(
                intern!(py, "drag_coef").clone().unbind(),
                Play::variant(self).drag_coef(),
                0.0,
                py,
            ),
            Arg::key_with_default(
                intern!(py, "frequency").clone().unbind(),
                Play::variant(self).frequency(),
                Frequency::ZERO,
                py,
            ),
            Arg::key_with_default(
                intern!(py, "phase").clone().unbind(),
                Play::variant(self).phase(),
                Phase::ZERO,
                py,
            ),
            Arg::key_with_default(
                intern!(py, "flexible").clone().unbind(),
                Play::variant(self).flexible(),
                false,
                py,
            ),
        ]
        .into_iter()
        .chain(Play::common_repr(self))
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
#[pyclass(module="bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
struct ShiftPhase;

impl ElementSubclass for ShiftPhase {
    type Variant = schedule::ShiftPhase;
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
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: ChannelId,
        phase: Phase,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::ShiftPhase::new(channel_id, phase)?;
        Ok((
            Self,
            Self::build_element(
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
    fn channel_id<'a>(slf: &'a Bound<Self>) -> &'a ChannelId {
        Self::variant(slf).channel_id()
    }

    #[getter]
    fn phase(slf: &Bound<Self>) -> Phase {
        Self::variant(slf).phase()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, ShiftPhase> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        [
            Arg::positional(ShiftPhase::variant(self).channel_id(), py),
            Arg::positional(ShiftPhase::variant(self).phase(), py),
        ]
        .into_iter()
        .chain(ShiftPhase::common_repr(self))
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
#[pyclass(module="bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
struct SetPhase;

impl ElementSubclass for SetPhase {
    type Variant = schedule::SetPhase;
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
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: ChannelId,
        phase: Phase,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::SetPhase::new(channel_id, phase)?;
        Ok((
            Self,
            Self::build_element(
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
    fn channel_id<'a>(slf: &'a Bound<Self>) -> &'a ChannelId {
        Self::variant(slf).channel_id()
    }

    #[getter]
    fn phase(slf: &Bound<Self>) -> Phase {
        Self::variant(slf).phase()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, SetPhase> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        [
            Arg::positional(SetPhase::variant(self).channel_id(), py),
            Arg::positional(SetPhase::variant(self).phase(), py),
        ]
        .into_iter()
        .chain(SetPhase::common_repr(self))
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
#[pyclass(module="bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
struct ShiftFreq;

impl ElementSubclass for ShiftFreq {
    type Variant = schedule::ShiftFreq;
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
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: ChannelId,
        frequency: Frequency,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::ShiftFreq::new(channel_id, frequency)?;
        Ok((
            Self,
            Self::build_element(
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
    fn channel_id<'a>(slf: &'a Bound<Self>) -> &'a ChannelId {
        Self::variant(slf).channel_id()
    }

    #[getter]
    fn frequency(slf: &Bound<Self>) -> Frequency {
        Self::variant(slf).frequency()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, ShiftFreq> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        [
            Arg::positional(ShiftFreq::variant(self).channel_id(), py),
            Arg::positional(ShiftFreq::variant(self).frequency(), py),
        ]
        .into_iter()
        .chain(ShiftFreq::common_repr(self))
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
#[pyclass(module="bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
struct SetFreq;

impl ElementSubclass for SetFreq {
    type Variant = schedule::SetFreq;
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
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id: ChannelId,
        frequency: Frequency,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::SetFreq::new(channel_id, frequency)?;
        Ok((
            Self,
            Self::build_element(
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
    fn channel_id<'a>(slf: &'a Bound<Self>) -> &'a ChannelId {
        Self::variant(slf).channel_id()
    }

    #[getter]
    fn frequency(slf: &Bound<Self>) -> Frequency {
        Self::variant(slf).frequency()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, SetFreq> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        [
            Arg::positional(SetFreq::variant(self).channel_id(), py),
            Arg::positional(SetFreq::variant(self).frequency(), py),
        ]
        .into_iter()
        .chain(SetFreq::common_repr(self))
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
#[pyclass(module="bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
struct SwapPhase;

impl ElementSubclass for SwapPhase {
    type Variant = schedule::SwapPhase;
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
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        channel_id1: ChannelId,
        channel_id2: ChannelId,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::SwapPhase::new(channel_id1, channel_id2);
        Ok((
            Self,
            Self::build_element(
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
    fn channel_id1<'a>(slf: &'a Bound<Self>) -> &'a ChannelId {
        Self::variant(slf).channel_id1()
    }

    #[getter]
    fn channel_id2<'a>(slf: &'a Bound<Self>) -> &'a ChannelId {
        Self::variant(slf).channel_id2()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, SwapPhase> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        [
            Arg::positional(SwapPhase::variant(self).channel_id1(), py),
            Arg::positional(SwapPhase::variant(self).channel_id2(), py),
        ]
        .into_iter()
        .chain(SwapPhase::common_repr(self))
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
#[pyclass(module="bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
struct Barrier;

impl ElementSubclass for Barrier {
    type Variant = schedule::Barrier;
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
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    fn new(
        channel_ids: Vec<ChannelId>,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::Barrier::new(channel_ids);
        Ok((
            Self,
            Self::build_element(
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
    fn channel_ids(slf: &Bound<Self>) -> Vec<ChannelId> {
        Self::variant(slf).channel_ids().to_vec()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, Barrier> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        Barrier::variant(self)
            .channel_ids()
            .iter()
            .map(|x| Arg::positional(x, py))
            .chain(Barrier::common_repr(self))
            .collect_vec()
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
#[pyclass(module="bosing",extends=Element, get_all, frozen)]
#[derive(Debug)]
struct Repeat {
    child: Py<Element>,
}

impl ElementSubclass for Repeat {
    type Variant = schedule::Repeat;
}

#[pymethods]
impl Repeat {
    #[new]
    #[pyo3(signature = (
        child,
        count,
        spacing=Time::ZERO,
        *,
        margin=None,
        alignment=None,
        phantom=false,
        duration=None,
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        child: Py<Element>,
        count: usize,
        spacing: Time,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
    ) -> PyResult<(Self, Element)> {
        let rust_child = child.get().0.clone();
        let variant = schedule::Repeat::new(rust_child, count).with_spacing(spacing)?;
        Ok((
            Self { child },
            Self::build_element(
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
    fn count(slf: &Bound<Self>) -> usize {
        Self::variant(slf).count()
    }

    #[getter]
    fn spacing(slf: &Bound<Self>) -> Time {
        Self::variant(slf).spacing()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, Repeat> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        [
            Arg::positional(&self.get().child, py),
            Arg::positional(Repeat::variant(self).count(), py),
            Arg::key_with_default(
                intern!(py, "spacing").clone().unbind(),
                Repeat::variant(self).spacing(),
                Time::ZERO,
                py,
            ),
        ]
        .into_iter()
        .chain(Repeat::common_repr(self))
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
#[pyclass(module = "bosing", frozen, eq)]
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
    fn convert(obj: &Bound<PyAny>) -> PyResult<Py<Self>> {
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(s) = obj.extract::<String>() {
            let direction = match s.as_str() {
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

fn extract_direction(obj: &Bound<PyAny>) -> PyResult<Direction> {
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
#[pyclass(module="bosing",extends=Element, get_all, frozen)]
#[derive(Debug)]
struct Stack {
    children: Vec<Py<Element>>,
}

impl ElementSubclass for Stack {
    type Variant = schedule::Stack;
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
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        children: Vec<Py<Element>>,
        direction: Option<&Bound<PyAny>>,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
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
            Self::build_element(
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
    fn with_children(slf: &Bound<Self>, children: Vec<Py<Element>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let rust_children = children.iter().map(|x| x.get().0.clone()).collect();
        let rust_base = &slf.downcast::<Element>()?.get().0;
        let common = rust_base.common.clone();
        let variant = Self::variant(slf).clone().with_children(rust_children);
        Py::new(
            py,
            (
                Self { children },
                Element(Arc::new(schedule::Element::new(common, variant))),
            ),
        )
    }

    #[getter]
    fn direction(slf: &Bound<Self>) -> Direction {
        Self::variant(slf).direction()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, Stack> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        self.get()
            .children
            .iter()
            .map(|x| Arg::positional(x, py))
            .chain([Arg::key_with_default(
                intern!(py, "direction").clone().unbind(),
                Stack::variant(self).direction().into_py(py),
                Direction::Backward.into_py(py),
                py,
            )])
            .chain(Stack::common_repr(self))
            .collect_vec()
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
#[pyclass(module = "bosing", get_all, frozen)]
#[derive(Debug)]
struct AbsoluteEntry {
    time: Time,
    element: Py<Element>,
}

impl AbsoluteEntry {
    fn clone_ref(&self, py: Python) -> Self {
        Self {
            time: self.time,
            element: self.element.clone_ref(py),
        }
    }
}

#[pymethods]
impl AbsoluteEntry {
    #[new]
    fn new(time: Time, element: Py<Element>) -> PyResult<Self> {
        if !time.value().is_finite() {
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
    fn convert(obj: &Bound<PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(element) = obj.extract() {
            return Py::new(py, AbsoluteEntry::new(Time::ZERO, element)?);
        }
        if let Ok((time, element)) = obj.extract() {
            return Py::new(py, AbsoluteEntry::new(time, element)?);
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to AbsoluteEntry",
        ))
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.borrow().to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(&self, py: Python) -> Vec<Arg> {
        self.to_rich_repr(py)
    }
}

impl RichRepr for AbsoluteEntry {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        repr_list!(self, py, time, element;;)
    }
}

impl<'py> FromPyObject<'py> for AbsoluteEntry {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let py = ob.py();
        let ob = ob.downcast_exact::<Self>()?.get();
        Ok(ob.clone_ref(py))
    }
}

fn extract_absolute_entry(obj: &Bound<PyAny>) -> PyResult<AbsoluteEntry> {
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
#[pyclass(module="bosing",extends=Element, frozen)]
#[derive(Debug)]
struct Absolute {
    children: Vec<AbsoluteEntry>,
}

impl ElementSubclass for Absolute {
    type Variant = schedule::Absolute;
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
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python,
        children: Vec<Py<PyAny>>,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
    ) -> PyResult<(Self, Element)> {
        let children: Vec<AbsoluteEntry> = children
            .into_iter()
            .map(|x| extract_absolute_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                Ok(schedule::AbsoluteEntry::new(element).with_time(x.time)?)
            })
            .collect::<PyResult<_>>()?;
        let variant = schedule::Absolute::new().with_children(rust_children);
        Ok((
            Self { children },
            Self::build_element(
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
    fn with_children(slf: &Bound<Self>, children: Vec<Py<PyAny>>) -> PyResult<Py<Self>> {
        let py = slf.py();
        let children: Vec<_> = children
            .into_iter()
            .map(|x| extract_absolute_entry(&x.into_bound(py)))
            .collect::<PyResult<_>>()?;
        let rust_children = children
            .iter()
            .map(|x| {
                let element = x.element.get().0.clone();
                Ok(schedule::AbsoluteEntry::new(element).with_time(x.time)?)
            })
            .collect::<PyResult<_>>()?;
        let rust_base = &slf.downcast::<Element>()?.get().0;
        let common = rust_base.common.clone();
        let variant = Self::variant(slf).clone().with_children(rust_children);
        Py::new(
            py,
            (
                Self { children },
                Element(Arc::new(schedule::Element::new(common, variant))),
            ),
        )
    }

    #[getter]
    fn children(slf: &Bound<Self>) -> Vec<AbsoluteEntry> {
        let py = slf.py();
        slf.get().children.iter().map(|x| x.clone_ref(py)).collect()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, Absolute> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        self.get()
            .children
            .iter()
            .map(|x| Arg::positional(x.clone_ref(py).into_py(py), py))
            .chain(Absolute::common_repr(self))
            .collect_vec()
    }
}

/// Unit of grid length.
///
/// The unit can be:
///
/// - Seconds: Fixed length in seconds.
/// - Auto: Auto length.
/// - Star: Ratio of the remaining duration.
#[pyclass(module = "bosing", frozen, eq)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridLengthUnit {
    Seconds,
    Auto,
    Star,
}

impl ToPyObject for GridLengthUnit {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        (*self).into_py(py)
    }
}

/// Length of a grid column.
///
/// :class:`GridLength` is used to specify the length of a grid column. The
/// length can be specified in seconds, as a fraction of the remaining duration,
/// or automatically.
#[pyclass(module = "bosing", get_all, frozen)]
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
        if !(value.is_finite() && value > 0.0) {
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
        if !(value.is_finite() && value >= 0.0) {
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
    fn convert(obj: &Bound<PyAny>) -> PyResult<Py<Self>> {
        let py = obj.py();
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(v) = obj.extract() {
            return Py::new(py, GridLength::fixed(v)?);
        }
        if let Ok(s) = obj.extract::<String>() {
            return Py::new(py, GridLength::from_str(&s)?);
        }
        Err(PyValueError::new_err(
            "Failed to convert the value to GridLength.",
        ))
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.borrow().to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(&self, py: Python) -> Vec<Arg> {
        self.to_rich_repr(py)
    }
}

impl RichRepr for GridLength {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        repr_list!(self, py, value, unit;;)
    }
}

impl GridLength {
    fn is_auto(&self) -> bool {
        self.unit == GridLengthUnit::Auto
    }

    fn is_star(&self) -> bool {
        self.unit == GridLengthUnit::Star
    }

    fn is_fixed(&self) -> bool {
        self.unit == GridLengthUnit::Seconds
    }
}

impl FromStr for GridLength {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "auto" {
            return Ok(GridLength::auto());
        }
        if s == "*" {
            return Ok(GridLength::star(1.0)?);
        }
        if let Some(v) = s.strip_suffix('*').and_then(|x| x.parse().ok()) {
            return Ok(GridLength::star(v)?);
        }
        if let Ok(v) = s.parse() {
            return Ok(GridLength::fixed(v)?);
        }
        Err(anyhow::anyhow!("Invalid GridLength string: {}", s))
    }
}

impl ToPyObject for GridLength {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone().into_py(py)
    }
}

fn extract_grid_length(obj: &Bound<PyAny>) -> PyResult<GridLength> {
    GridLength::convert(obj).and_then(|x| x.extract(obj.py()))
}

/// A child element in a grid layout.
///
/// Args:
///     element (Element): Child element.
///     column (int): Column index.
///     span (int): Column span.
#[pyclass(module = "bosing", get_all, frozen)]
#[derive(Debug)]
struct GridEntry {
    element: Py<Element>,
    column: usize,
    span: usize,
}

impl GridEntry {
    fn clone_ref(&self, py: Python) -> Self {
        Self {
            element: self.element.clone_ref(py),
            column: self.column,
            span: self.span,
        }
    }
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
    fn convert(obj: &Bound<PyAny>) -> PyResult<Py<Self>> {
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

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.borrow().to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(&self, py: Python) -> Vec<Arg> {
        self.to_rich_repr(py)
    }
}

impl RichRepr for GridEntry {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        repr_list!(self, py, element;; column=0, span=1)
    }
}

impl<'py> FromPyObject<'py> for GridEntry {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let py = ob.py();
        let ob = ob.downcast_exact::<Self>()?.get();
        Ok(ob.clone_ref(py))
    }
}

fn extract_grid_entry(obj: &Bound<PyAny>) -> PyResult<GridEntry> {
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
#[pyclass(module="bosing",extends=Element, frozen)]
#[derive(Debug)]
struct Grid {
    children: Vec<GridEntry>,
}

impl ElementSubclass for Grid {
    type Variant = schedule::Grid;
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
        max_duration=Time::INFINITY,
        min_duration=Time::ZERO,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python,
        children: Vec<Py<PyAny>>,
        columns: Vec<Py<PyAny>>,
        margin: Option<&Bound<PyAny>>,
        alignment: Option<&Bound<PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
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
                    .expect("Should be checked in GridEntry::new")
            })
            .collect();
        let variant = schedule::Grid::new()
            .with_children(rust_children)
            .with_columns(columns);
        Ok((
            Self { children },
            Self::build_element(
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
    fn with_children(slf: &Bound<Self>, children: Vec<Py<PyAny>>) -> PyResult<Py<Self>> {
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
                    .expect("Should be checked in GridEntry::new")
            })
            .collect();
        let rust_base = &slf.downcast::<Element>()?.get().0;
        let common = rust_base.common.clone();
        let variant = Self::variant(slf).clone().with_children(rust_children);
        Py::new(
            py,
            (
                Self { children },
                Element(Arc::new(schedule::Element::new(common, variant))),
            ),
        )
    }

    #[getter]
    fn columns(slf: &Bound<Self>) -> Vec<GridLength> {
        Self::variant(slf).columns().to_vec()
    }

    #[getter]
    fn children(slf: &Bound<Self>) -> Vec<GridEntry> {
        let py = slf.py();
        slf.get().children.iter().map(|x| x.clone_ref(py)).collect()
    }

    fn __repr__(slf: &Bound<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().qualname()?;
        slf.to_repr(cls_name, slf.py())
    }

    fn __rich_repr__(slf: &Bound<Self>) -> Vec<Arg> {
        slf.to_rich_repr(slf.py())
    }
}

impl<'py> RichRepr for Bound<'py, Grid> {
    fn repr(&self, py: Python) -> impl IntoIterator<Item = Arg> {
        self.get()
            .children
            .iter()
            .map(|x| Arg::positional(x.clone_ref(py).into_py(py), py))
            .chain([Arg::keyword(
                intern!(py, "columns").clone().unbind(),
                Grid::variant(self).columns(),
                py,
            )])
            .chain(Grid::common_repr(self))
            .collect_vec()
    }
}

type ChannelWaveforms = HashMap<ChannelId, Py<PyArray2<f64>>>;
type ChannelStates = HashMap<ChannelId, Py<OscState>>;
type ChannelPulses = HashMap<ChannelId, PulseList>;

#[pyfunction]
#[pyo3(signature = (
    *,
    channels,
    shapes,
    schedule,
    time_tolerance,
    amp_tolerance,
    allow_oversize,
    crosstalk,
))]
#[allow(clippy::too_many_arguments)]
fn generate_waveforms(
    py: Python,
    channels: HashMap<ChannelId, Channel>,
    shapes: HashMap<ShapeId, Py<Shape>>,
    schedule: Bound<Element>,
    time_tolerance: Time,
    amp_tolerance: Amplitude,
    allow_oversize: bool,
    crosstalk: Option<(PyArrayLike2<f64, AllowTypeChange>, Vec<ChannelId>)>,
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

#[pyfunction]
#[pyo3(signature = (
    *,
    channels,
    shapes,
    schedule,
    time_tolerance,
    amp_tolerance,
    allow_oversize,
    crosstalk,
    states,
))]
#[allow(clippy::too_many_arguments)]
fn generate_waveforms_with_states(
    py: Python,
    channels: HashMap<ChannelId, Channel>,
    shapes: HashMap<ShapeId, Py<Shape>>,
    schedule: Bound<Element>,
    time_tolerance: Time,
    amp_tolerance: Amplitude,
    allow_oversize: bool,
    crosstalk: Option<(PyArrayLike2<f64, AllowTypeChange>, Vec<ChannelId>)>,
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
        states,
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

fn build_pulse_lists(
    schedule: Bound<Element>,
    channels: &HashMap<ChannelId, Channel>,
    shapes: &HashMap<ShapeId, Py<Shape>>,
    time_tolerance: Time,
    amp_tolerance: Amplitude,
    allow_oversize: bool,
    states: Option<ChannelStates>,
) -> PyResult<(ChannelPulses, ChannelStates)> {
    let py = schedule.py();
    let mut executor = Executor::new(amp_tolerance, time_tolerance, allow_oversize);
    for (n, c) in channels {
        let osc = match &states {
            Some(states) => {
                let state = states
                    .get(n)
                    .ok_or_else(|| PyValueError::new_err(format!("No state for channel: {}", n)))?;
                let state = state.bind(py);
                state.extract::<OscState>()?.into()
            }
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
    py: Python,
    channels: &HashMap<ChannelId, Channel>,
    pulse_lists: ChannelPulses,
    crosstalk: Option<(PyArrayLike2<f64, AllowTypeChange>, Vec<ChannelId>)>,
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
    if let Some((crosstalk, names)) = &crosstalk {
        sampler.set_crosstalk(crosstalk.as_array(), names.clone());
    }
    py.allow_threads(|| sampler.sample(time_tolerance))?;
    Ok(waveforms)
}

fn post_process(w: &mut ArrayViewMut2<f64>, c: &Channel) {
    let iq_matrix = c.iq_matrix.as_ref().map(|x| x.view());
    let offset = c.offset.as_ref().map(|x| x.view());
    let iir = c.iir.as_ref().map(|x| x.view());
    let fir = c.fir.as_ref().map(|x| x.view());
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

#[pymodule]
fn _bosing(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Absolute>()?;
    m.add_class::<AbsoluteEntry>()?;
    m.add_class::<Alignment>()?;
    m.add_class::<Barrier>()?;
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
