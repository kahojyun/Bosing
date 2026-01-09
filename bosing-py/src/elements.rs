mod absolute;
mod grid;
mod stack;

use std::{borrow::Borrow as _, fmt::Debug, sync::Arc};

use bosing::schedule::{
    self, ElementCommon, ElementCommonBuilder, ElementRef, ElementVariant, Measure as _,
};
use pyo3::{exceptions::PyValueError, prelude::*, pybacked::PyBackedStr, types::DerefToPyAny};

use crate::{
    push_repr,
    types::{Amplitude, ChannelId, Frequency, Label, Phase, ShapeId, Time},
};

use super::{
    plot,
    repr::{Arg, Rich},
};

pub use self::{
    absolute::{Absolute, Entry as AbsoluteEntry},
    grid::{Entry as GridEntry, Grid, Length as GridLength, LengthUnit as GridLengthUnit},
    stack::{Direction, Stack},
};

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
pub enum Alignment {
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
    ///
    /// Returns:
    ///     Alignment: The converted value.
    ///
    /// Raises:
    ///     ValueError: If the value cannot be converted to Alignment.
    #[staticmethod]
    fn convert(obj: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        if let Ok(slf) = obj.extract() {
            return Ok(slf);
        }
        if let Ok(s) = obj.extract::<PyBackedStr>() {
            let alignment = match &*s {
                "end" => Some(Self::End),
                "start" => Some(Self::Start),
                "center" => Some(Self::Center),
                "stretch" => Some(Self::Stretch),
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

impl From<Alignment> for schedule::Alignment {
    fn from(value: Alignment) -> Self {
        match value {
            Alignment::End => Self::End,
            Alignment::Start => Self::Start,
            Alignment::Center => Self::Center,
            Alignment::Stretch => Self::Stretch,
        }
    }
}

impl From<schedule::Alignment> for Alignment {
    fn from(value: schedule::Alignment) -> Self {
        match value {
            schedule::Alignment::End => Self::End,
            schedule::Alignment::Start => Self::Start,
            schedule::Alignment::Center => Self::Center,
            schedule::Alignment::Stretch => Self::Stretch,
        }
    }
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
///         ``0``.
///     alignment (str | Alignment): Alignment of the element. The value can
///         be :class:`Alignment` or one of 'end', 'start', 'center', 'stretch'.
///         Defaults to :attr:`Alignment.End`.
///     phantom (bool): Whether the element is a phantom element and should not
///         add to waveforms. Defaults to ``False``.
///     duration (float): Duration of the element. Defaults to ``None``.
///     max_duration (float): Maximum duration of the element. Defaults to
///         ``inf``.
///     min_duration (float): Minimum duration of the element. Defaults to ``0``.
///     label (str | None): Label of the element. Defaults to ``None``.
#[pyclass(module = "bosing", subclass, frozen)]
#[derive(Debug, Clone)]
pub struct Element(pub(super) ElementRef);

#[pymethods]
impl Element {
    #[getter]
    fn margin(&self) -> (Time, Time) {
        let margin = self.0.common.margin();
        (margin.0.into(), margin.1.into())
    }

    #[getter]
    fn alignment(&self) -> Alignment {
        self.0.common.alignment().into()
    }

    #[getter]
    fn phantom(&self) -> bool {
        self.0.common.phantom()
    }

    #[getter]
    fn duration(&self) -> Option<Time> {
        self.0.common.duration().map(Into::into)
    }

    #[getter]
    fn max_duration(&self) -> Time {
        self.0.common.max_duration().into()
    }

    #[getter]
    fn min_duration(&self) -> Time {
        self.0.common.min_duration().into()
    }

    #[getter]
    fn label(&self) -> Option<Label> {
        self.0.common.label().cloned().map(Into::into)
    }

    /// Measure the minimum total duration required by the element.
    ///
    /// This value includes both inner `duration` and outer `margin` of the element.
    ///
    /// This value is a *minimum* total duration wanted by the element. If the element is a child
    /// of other element, the final total duration will be determined by `alignment` option and
    /// parent container type.
    fn measure(&self) -> Time {
        self.0.measure().into()
    }

    /// Plot arrange result with the element as root.
    ///
    /// Args:
    ///     ax (matplotlib.axes.Axes | None): Axes to plot. If ``None``, `matplotlib.pyplot.gca` is
    ///         used.
    ///     channels (Sequence[str] | None): Channels to plot. If ``None``, all channels are
    ///         plotted.
    ///     max_depth (int): Maximum depth to plot. Defaults to ``5``.
    ///     show_label (bool): Whether to show label of elements. Defaults to ``True``.
    ///
    /// Returns:
    ///     matplotlib.axes.Axes: Axes with the plot.
    #[pyo3(signature = (ax=None, *, channels=None, max_depth=5, show_label=true))]
    fn plot(
        &self,
        py: Python<'_>,
        ax: Option<Py<PyAny>>,
        channels: Option<Vec<ChannelId>>,
        max_depth: usize,
        show_label: bool,
    ) -> PyResult<Py<PyAny>> {
        plot::element(py, self.0.clone(), ax, channels, max_depth, show_label)
    }
}

trait ElementSubclass: Sized + DerefToPyAny
where
    for<'a> &'a Self::Variant: TryFrom<&'a ElementVariant, Error: Debug>,
{
    type Variant: Into<ElementVariant>;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg>;

    fn inner<'a>(slf: &'a Bound<'_, Self>) -> &'a ElementRef {
        slf.cast::<Element>()
            .expect("Self should be a subclass of Element")
            .get()
            .0
            .borrow()
    }

    fn common<'a>(slf: &'a Bound<'_, Self>) -> &'a ElementCommon {
        Self::inner(slf).common.borrow()
    }

    fn variant<'a>(slf: &'a Bound<'_, Self>) -> &'a Self::Variant {
        Self::inner(slf)
            .variant
            .borrow()
            .try_into()
            .expect("Element should have a valid variant")
    }

    #[expect(clippy::too_many_arguments)]
    fn build_element(
        variant: Self::Variant,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<Element> {
        fn extract_alignment(obj: &Bound<'_, PyAny>) -> PyResult<Alignment> {
            Alignment::convert(obj).and_then(|x| x.extract(obj.py()).map_err(Into::into))
        }

        fn extract_margin(obj: &Bound<'_, PyAny>) -> PyResult<(Time, Time)> {
            if let Ok(v) = obj.extract() {
                return Ok((v, v));
            }
            if let Ok((v1, v2)) = obj.extract() {
                return Ok((v1, v2));
            }
            let msg = "Failed to convert the value to (float, float).";
            Err(PyValueError::new_err(msg))
        }

        let mut builder = ElementCommonBuilder::new();
        if let Some(obj) = margin {
            let margin = extract_margin(obj)?;
            builder.margin((margin.0.into(), margin.1.into()));
        }
        if let Some(obj) = alignment {
            builder.alignment(extract_alignment(obj)?.into());
        }
        builder
            .phantom(phantom)
            .duration(duration.map(Into::into))
            .max_duration(max_duration.into())
            .min_duration(min_duration.into())
            .label(label.map(Into::into));
        let common = builder.build()?;
        Ok(Element(Arc::new(schedule::Element::new(common, variant))))
    }
}

impl<T> Rich for T
where
    T: ElementSubclass,
    for<'a> &'a T::Variant: TryFrom<&'a ElementVariant, Error: Debug>,
{
    fn repr(slf: &Bound<'_, Self>) -> impl Iterator<Item = Arg> {
        let mut res = Self::repr(slf);
        let py = slf.py();
        let slf = Self::common(slf);
        let margin = slf.margin();
        push_repr!(
            res,
            py,
            "margin",
            (margin.0.into(), margin.1.into()),
            (Time::ZERO, Time::ZERO)
        );
        push_repr!(
            res,
            py,
            "alignment",
            Alignment::from(slf.alignment()),
            Alignment::End
        );
        push_repr!(res, py, "phantom", slf.phantom(), false);
        let duration: Option<Time> = slf.duration().map(Into::into);
        push_repr!(res, py, "duration", duration, None);
        push_repr!(
            res,
            py,
            "max_duration",
            slf.max_duration().into(),
            Time::INFINITY
        );
        push_repr!(
            res,
            py,
            "min_duration",
            slf.min_duration().into(),
            Time::ZERO
        );
        let label: Option<Label> = slf.label().cloned().map(Into::into);
        push_repr!(res, py, "label", label, None);
        res.into_iter()
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
///     plateau (float): Plateau length of the pulse. Defaults to ``0``.
///     drag_coef (float): Drag coefficient of the pulse. If the pulse is a
///         rectangular pulse, the drag coefficient is ignored. Defaults to ``0``.
///     frequency (float): Additional frequency of the pulse on top of channel
///         base frequency and frequency shift. Defaults to ``0``.
///     phase (float): Additional phase of the pulse in **cycles**. Defaults to
///         ``0``.
///     flexible (bool): Whether the pulse has flexible plateau length. Defaults
///         to ``False``.
#[pyclass(module="bosing._bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
pub struct Play;

impl ElementSubclass for Play {
    type Variant = schedule::Play;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        push_repr!(res, py, Self::channel_id(slf));
        push_repr!(res, py, Self::shape_id(slf));
        push_repr!(res, py, Self::amplitude(slf));
        push_repr!(res, py, Self::width(slf));
        push_repr!(res, py, "plateau", Self::plateau(slf), Time::ZERO);
        push_repr!(res, py, "drag_coef", Self::drag_coef(slf), 0.0);
        push_repr!(res, py, "frequency", Self::frequency(slf), Frequency::ZERO);
        push_repr!(res, py, "phase", Self::phase(slf), Phase::ZERO);
        push_repr!(res, py, "flexible", Self::flexible(slf), false);
        res
    }
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
        label=None,
    ))]
    #[expect(clippy::too_many_arguments)]
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
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::Play::new(
            channel_id.into(),
            shape_id.map(Into::into),
            amplitude.into(),
            width.into(),
        )?
        .with_plateau(plateau.into())?
        .with_drag_coef(drag_coef)?
        .with_frequency(frequency.into())?
        .with_phase(phase.into())?
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
                label,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> ChannelId {
        Self::variant(slf).channel_id().clone().into()
    }

    #[getter]
    fn shape_id(slf: &Bound<'_, Self>) -> Option<ShapeId> {
        Self::variant(slf).shape_id().cloned().map(Into::into)
    }

    #[getter]
    fn amplitude(slf: &Bound<'_, Self>) -> Amplitude {
        Self::variant(slf).amplitude().into()
    }

    #[getter]
    fn width(slf: &Bound<'_, Self>) -> Time {
        Self::variant(slf).width().into()
    }

    #[getter]
    fn plateau(slf: &Bound<'_, Self>) -> Time {
        Self::variant(slf).plateau().into()
    }

    #[getter]
    fn drag_coef(slf: &Bound<'_, Self>) -> f64 {
        Self::variant(slf).drag_coef()
    }

    #[getter]
    fn frequency(slf: &Bound<'_, Self>) -> Frequency {
        Self::variant(slf).frequency().into()
    }

    #[getter]
    fn phase(slf: &Bound<'_, Self>) -> Phase {
        Self::variant(slf).phase().into()
    }

    #[getter]
    fn flexible(slf: &Bound<'_, Self>) -> bool {
        Self::variant(slf).flexible()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
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
#[pyclass(module="bosing._bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
pub struct ShiftPhase;

impl ElementSubclass for ShiftPhase {
    type Variant = schedule::ShiftPhase;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        push_repr!(res, py, Self::channel_id(slf));
        push_repr!(res, py, Self::phase(slf));
        res
    }
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
        label=None,
    ))]
    #[expect(clippy::too_many_arguments)]
    fn new(
        channel_id: ChannelId,
        phase: Phase,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::ShiftPhase::new(channel_id.into(), phase.into())?;
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
                label,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> ChannelId {
        Self::variant(slf).channel_id().clone().into()
    }

    #[getter]
    fn phase(slf: &Bound<'_, Self>) -> Phase {
        Self::variant(slf).phase().into()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
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
#[pyclass(module="bosing._bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
pub struct SetPhase;

impl ElementSubclass for SetPhase {
    type Variant = schedule::SetPhase;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        push_repr!(res, py, Self::channel_id(slf));
        push_repr!(res, py, Self::phase(slf));
        res
    }
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
        label=None,
    ))]
    #[expect(clippy::too_many_arguments)]
    fn new(
        channel_id: ChannelId,
        phase: Phase,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::SetPhase::new(channel_id.into(), phase.into())?;
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
                label,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> ChannelId {
        Self::variant(slf).channel_id().clone().into()
    }

    #[getter]
    fn phase(slf: &Bound<'_, Self>) -> Phase {
        Self::variant(slf).phase().into()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
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
#[pyclass(module="bosing._bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
pub struct ShiftFreq;

impl ElementSubclass for ShiftFreq {
    type Variant = schedule::ShiftFreq;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        push_repr!(res, py, Self::channel_id(slf));
        push_repr!(res, py, Self::frequency(slf));
        res
    }
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
        label=None,
    ))]
    #[expect(clippy::too_many_arguments)]
    fn new(
        channel_id: ChannelId,
        frequency: Frequency,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::ShiftFreq::new(channel_id.into(), frequency.into())?;
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
                label,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> ChannelId {
        Self::variant(slf).channel_id().clone().into()
    }

    #[getter]
    fn frequency(slf: &Bound<'_, Self>) -> Frequency {
        Self::variant(slf).frequency().into()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
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
#[pyclass(module="bosing._bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
pub struct SetFreq;

impl ElementSubclass for SetFreq {
    type Variant = schedule::SetFreq;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        push_repr!(res, py, Self::channel_id(slf));
        push_repr!(res, py, Self::frequency(slf));
        res
    }
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
        label=None,
    ))]
    #[expect(clippy::too_many_arguments)]
    fn new(
        channel_id: ChannelId,
        frequency: Frequency,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::SetFreq::new(channel_id.into(), frequency.into())?;
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
                label,
            )?,
        ))
    }

    #[getter]
    fn channel_id(slf: &Bound<'_, Self>) -> ChannelId {
        Self::variant(slf).channel_id().clone().into()
    }

    #[getter]
    fn frequency(slf: &Bound<'_, Self>) -> Frequency {
        Self::variant(slf).frequency().into()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
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
#[pyclass(module="bosing._bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
pub struct SwapPhase;

impl ElementSubclass for SwapPhase {
    type Variant = schedule::SwapPhase;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        push_repr!(res, py, Self::channel_id1(slf));
        push_repr!(res, py, Self::channel_id2(slf));
        res
    }
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
        label=None,
    ))]
    #[expect(clippy::too_many_arguments)]
    fn new(
        channel_id1: ChannelId,
        channel_id2: ChannelId,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let variant = schedule::SwapPhase::new(channel_id1.into(), channel_id2.into());
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
                label,
            )?,
        ))
    }

    #[getter]
    fn channel_id1(slf: &Bound<'_, Self>) -> ChannelId {
        Self::variant(slf).channel_id1().clone().into()
    }

    #[getter]
    fn channel_id2(slf: &Bound<'_, Self>) -> ChannelId {
        Self::variant(slf).channel_id2().clone().into()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
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
#[pyclass(module="bosing._bosing",extends=Element, frozen)]
#[derive(Debug, Clone)]
pub struct Barrier;

impl ElementSubclass for Barrier {
    type Variant = schedule::Barrier;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let py = slf.py();
        Self::variant(slf)
            .channel_ids()
            .iter()
            .map(|x| Arg::positional(ChannelId::from(x.clone()), py))
            .collect()
    }
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
        label=None,
    ))]
    #[expect(clippy::too_many_arguments)]
    fn new(
        channel_ids: Vec<ChannelId>,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let channel_ids: Vec<_> = channel_ids.into_iter().map(Into::into).collect();
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
                label,
            )?,
        ))
    }

    #[getter]
    fn channel_ids(slf: &Bound<'_, Self>) -> Vec<ChannelId> {
        Self::variant(slf)
            .channel_ids()
            .iter()
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
    }
}

/// A repeat element.
///
/// Repeat the child element multiple times with a spacing between repetitions.
///
/// Args:
///     child (Element): Child element to repeat.
///     count (int): Number of repetitions.
///     spacing (float): Spacing between repetitions. Defaults to ``0``.
#[pyclass(module="bosing._bosing",extends=Element, get_all, frozen)]
#[derive(Debug)]
pub struct Repeat {
    child: Py<Element>,
}

impl ElementSubclass for Repeat {
    type Variant = schedule::Repeat;

    fn repr(slf: &Bound<'_, Self>) -> Vec<Arg> {
        let mut res = Vec::new();
        let py = slf.py();
        push_repr!(res, py, &slf.get().child);
        push_repr!(res, py, Self::count(slf));
        push_repr!(res, py, "spacing", Self::spacing(slf), Time::ZERO);
        res
    }
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
        label=None,
    ))]
    #[expect(clippy::too_many_arguments)]
    fn new(
        child: Py<Element>,
        count: usize,
        spacing: Time,
        margin: Option<&Bound<'_, PyAny>>,
        alignment: Option<&Bound<'_, PyAny>>,
        phantom: bool,
        duration: Option<Time>,
        max_duration: Time,
        min_duration: Time,
        label: Option<Label>,
    ) -> PyResult<(Self, Element)> {
        let rust_child = child.get().0.clone();
        let variant = schedule::Repeat::new(rust_child, count).with_spacing(spacing.into())?;
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
                label,
            )?,
        ))
    }

    #[getter]
    fn count(slf: &Bound<'_, Self>) -> usize {
        Self::variant(slf).count()
    }

    #[getter]
    fn spacing(slf: &Bound<'_, Self>) -> Time {
        Self::variant(slf).spacing().into()
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::to_repr(slf)
    }

    fn __rich_repr__(slf: &Bound<'_, Self>) -> Vec<Arg> {
        Self::to_rich_repr(slf)
    }
}
