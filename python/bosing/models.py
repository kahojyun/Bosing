"""Data models for the Bosing service."""

import enum as _enum
import math as _math
import typing as _typing

import attrs as _attrs
import msgpack as _msgpack


@_attrs.frozen
class MsgObject:
    """Base class for all message objects.

    .. note::
        The order of the fields must be the same as the order of the fields in the
        corresponding data class in the server.
    """

    @property
    def data(self) -> tuple:
        """The data of the message object to be serialized."""
        return _attrs.astuple(self, recurse=False)

    def packb(self) -> bytes:
        """Serialize the message object to bytes in msgpack format."""

        def encode(obj: _typing.Union[MsgObject, _enum.Enum]):
            if isinstance(obj, MsgObject):
                return obj.data
            if isinstance(obj, _enum.Enum):
                return obj.value
            raise TypeError(f"Cannot encode object of type {type(obj)}")

        return _msgpack.packb(self, default=encode)  # type: ignore


@_attrs.frozen
class UnionObject(MsgObject):
    """Base class for all union objects.

    A union object is a message object that can be one of several types.
    """

    TYPE_ID: _typing.ClassVar[int]

    @property
    def data(self) -> tuple:
        return (self.TYPE_ID, super().data)


@_attrs.frozen
class Biquad(MsgObject):
    """A biquad filter."""

    b0: float
    b1: float
    b2: float
    a1: float
    a2: float


@_attrs.frozen
class IqCali(MsgObject):
    """IQ calibration data.

    The calibration data consists of a 2x2 transformation matrix and an 2x1 offset
    vector. The transformation matrix is applied first, followed by the offset vector.

    .. math::
        \\begin{pmatrix}
            I_{out} \\\\
            Q_{out}
        \\end{pmatrix} =
        \\begin{pmatrix}
            a & b \\\\
            c & d
        \\end{pmatrix}
        \\begin{pmatrix}
            I_{in} \\\\
            Q_{in}
        \\end{pmatrix} +
        \\begin{pmatrix}
            i_{offset} \\\\
            q_{offset}
        \\end{pmatrix}
    """

    a: float
    b: float
    c: float
    d: float
    i_offset: float = 0
    q_offset: float = 0


@_attrs.frozen
class Channel(MsgObject):
    """Channel configuration."""

    name: str
    base_freq: float
    sample_rate: float
    length: int
    delay: float = 0
    align_level: int = -10
    iq_calibration: _typing.Optional[IqCali] = None
    iir: _typing.List[Biquad] = _attrs.field(factory=list, converter=list)
    fir: _typing.List[float] = _attrs.field(factory=list, converter=list)


@_attrs.frozen
class Options(MsgObject):
    """Various options for waveform generation.

    :param time_tolerance: The time tolerance of the scheduler.
    :param amp_tolerance: The amplitude tolerance in waveform calculation.
    :param phase_tolerance: The phase tolerance in waveform calculation.
    :param allow_oversize: Whether to allow arranging schedules with duration shorter than desired.
    """

    time_tolerance: float = 1e-12
    amp_tolerance: float = 0.1 / 2**16
    phase_tolerance: float = 1e-4
    allow_oversize: bool = False


class Alignment(_enum.Enum):
    """Alignment of a schedule element."""

    END = 0
    START = 1
    CENTER = 2
    STRETCH = 3
    """Stretch to fill parent element."""


def _convert_margin(
    margin: _typing.Union[float, _typing.Tuple[float, float]]
) -> _typing.Tuple[float, float]:
    if not isinstance(margin, tuple):
        margin = (margin, margin)
    return margin


def _convert_alignment(
    alignment: _typing.Union[
        _typing.Literal["end", "start", "center", "stretch"], Alignment
    ]
) -> Alignment:
    if isinstance(alignment, str):
        return Alignment[alignment.upper()]
    return alignment


@_attrs.frozen
class Shape(UnionObject):
    """Base class for shapes."""


@_attrs.frozen
class Hann(Shape):
    """A Hann shape."""

    TYPE_ID = 0


@_attrs.frozen
class Interp(Shape):
    """An interpolated shape."""

    TYPE_ID = 1

    x_array: _typing.List[float] = _attrs.field(converter=list)
    y_array: _typing.List[float] = _attrs.field(converter=list)


@_attrs.frozen
class Element(UnionObject):
    """Base class for schedule elements.

    A schedule element is a node in the tree structure of a schedule similar to
    HTML elements. The design is inspired by `XAML in WPF / WinUI
    <https://learn.microsoft.com/en-us/windows/apps/design/layout/layouts-with-xaml>`_

    When :attr:`duration`, :attr:`max_duration`, and :attr:`min_duration` are
    conflicting, the priority is as follows:

    1. :attr:`min_duration`
    2. :attr:`max_duration`
    3. :attr:`duration`

    :param margin: The margin of the element. If a single value is given, it is
        used for both sides. Default to 0.
    :type margin: float | tuple[float, float]
    :param alignment: The alignment of the element. Default to
        :attr:`Alignment.END`.
    :param visibility: Whether the element has effect on the output. Default to
        ``True``.
    :param duration: Requested duration of the element. If ``None``, the actual
        duration is determined by the measuring and arranging process. Default
        to ``None``.
    :param max_duration: Maximum duration of the element. Default to infinity.
    :param min_duration: Minimum duration of the element. Default to 0.
    """

    margin: _typing.Tuple[float, float] = _attrs.field(
        kw_only=True, default=(0, 0), converter=_convert_margin
    )
    alignment: Alignment = _attrs.field(
        kw_only=True, default=Alignment.END, converter=_convert_alignment
    )
    visibility: bool = _attrs.field(kw_only=True, default=True)
    duration: _typing.Optional[float] = _attrs.field(kw_only=True, default=None)
    max_duration: float = _attrs.field(kw_only=True, default=_math.inf)
    min_duration: float = _attrs.field(kw_only=True, default=0)


@_attrs.frozen
class Play(Element):
    """A pulse play element.

    If :attr:`flexible` is set to ``True`` and :attr:`alignment` is set to
    :attr:`Alignment.STRETCH`, the plateau of the pulse is stretched to fill the
    parent element.

    :param channel_id: Target channel ID.
    :param amplitude: The amplitude of the pulse.
    :param shape_id: The shape ID of the pulse.
    :param width: The width of the pulse.
    :param plateau: The plateau of the pulse. Default to 0.
    :param drag_coef: The drag coefficient of the pulse. Default to 0.
    :param frequency: Additional frequency of the pulse on top of channel base
        frequency and frequency shift. Default to 0.
    :param phase: Additional phase of the pulse in **cycles**. Default to 0.
    :param flexible: Whether the pulse can be shortened or extended. Default to
        ``False``.
    """

    TYPE_ID = 0

    channel_id: int
    amplitude: float
    shape_id: int
    width: float
    plateau: float = _attrs.field(kw_only=True, default=0)
    drag_coef: float = _attrs.field(kw_only=True, default=0)
    frequency: float = _attrs.field(kw_only=True, default=0)
    phase: float = _attrs.field(kw_only=True, default=0)
    flexible: bool = _attrs.field(kw_only=True, default=False)


@_attrs.frozen
class ShiftPhase(Element):
    """A phase shift element.

    :param channel_id: Target channel ID.
    :param phase: Delta phase in **cycles**.
    """

    TYPE_ID = 1

    channel_id: int
    phase: float


@_attrs.frozen
class SetPhase(Element):
    """A phase set element.

    Given the base frequency :math:`f`, the frequency shift :math:`\\Delta f`,
    the time :math:`t`, and the phase offset :math:`\\phi_0`, the phase is
    defined as

    .. math::

        \\phi(t) = (f + \\Delta f) t + \\phi_0

    :class:`SetPhase` sets the phase offset :math:`\\phi_0` such that
    :math:`\\phi(t)` is equal to the given phase.

    :param channel_id: Target channel ID.
    :param phase: Target phase in **cycles**.
    """

    TYPE_ID = 2

    channel_id: int
    phase: float


@_attrs.frozen
class ShiftFreq(Element):
    """A frequency shift element.

    Additional frequency shift on top of the channel cumulative frequency shift.
    Phase offset will be adjusted accordingly such that the phase is continuous
    at the shift point.

    :param channel_id: Target channel ID.
    :param frequency: Delta frequency.
    """

    TYPE_ID = 3

    channel_id: int
    frequency: float


@_attrs.frozen
class SetFreq(Element):
    """A frequency set element.

    Set the channel frequency shift to the target frequency. Phase offset will
    be adjusted accordingly such that the phase is continuous at the shift point.

    :param channel_id: Target channel ID.
    :param frequency: Target frequency.
    """

    TYPE_ID = 4

    channel_id: int
    frequency: float


@_attrs.frozen
class SwapPhase(Element):
    """A phase swap element.

    This instruction swaps carrier phases between two target channels at the
    scheduled time point. Carrier phase is defined as

    .. math::
        \\phi(t) = (f + \\Delta f) t + \\phi_0

    where :math:`f` is the base frequency, :math:`\\Delta f` is the frequency
    shift, :math:`t` is the time, and :math:`\\phi_0` is the phase offset.

    :param channel_id1: Target channel ID 1.
    :param channel_id2: Target channel ID 2.
    """

    TYPE_ID = 5

    channel_id1: int
    channel_id2: int


@_attrs.frozen
class Barrier(Element):
    """A barrier element.

    A barrier element is a zero-duration no-op element. Useful for aligning
    elements on different channels in :class:`Stack`.

    If :attr:`channel_ids` is empty, the barrier is applied to
    all channels in its parent element.

    :param channel_ids: Target channel IDs. Default to empty.
    """

    TYPE_ID = 6

    channel_ids: _typing.List[int] = _attrs.field(converter=list, factory=list)


@_attrs.frozen
class Repeat(Element):
    """A repeated schedule element.

    :param child: The repeated element.
    :param count: The number of repetitions.
    :param spacing: The spacing between repeated elements. Default to 0.
    """

    TYPE_ID = 7

    child: Element
    count: int
    spacing: float = _attrs.field(kw_only=True, default=0)


class ArrangeDirection(_enum.Enum):
    """Direction of arrangement."""

    BACKWARDS = 0
    """Arrange from the end of the schedule."""
    FORWARDS = 1
    """Arrange from the start of the schedule."""


def _convert_direction(
    direction: _typing.Union[_typing.Literal["backwards", "forwards"], ArrangeDirection]
) -> ArrangeDirection:
    if isinstance(direction, str):
        return ArrangeDirection[direction.upper()]
    return direction


@_attrs.frozen
class Stack(Element):
    """Layout child elements in one direction.

    The child elements are arranged in one direction. The direction can be
    forwards or backwards.

    Child elements with no common channel are arranged in parallel.
    :class:`Barrier` can be used to synchronize multiple channels.

    :param children: Child elements.
    :param direction: The direction of arrangement.
    """

    TYPE_ID = 8

    children: _typing.List[Element] = _attrs.field(converter=list, factory=list)
    direction: ArrangeDirection = _attrs.field(
        kw_only=True, default=ArrangeDirection.BACKWARDS, converter=_convert_direction
    )

    def with_children(self, *children: Element) -> "Stack":
        """Create a new stack with different children.

        :param children: The new children.
        :return: The new stack.
        """
        return _attrs.evolve(self, children=children)


@_attrs.frozen
class AbsoluteEntry(MsgObject):
    """An entry in the absolute schedule.

    :param time: Time relative to the start of the absolute schedule.
    :param element: The child element.
    """

    time: float
    element: Element

    @classmethod
    def from_tuple(
        cls, obj: _typing.Union[Element, _typing.Tuple[float, Element], "AbsoluteEntry"]
    ) -> "AbsoluteEntry":
        """Create an absolute entry from a tuple.

        :param obj: The object to be converted.
        :return: The converted object.
        """
        if isinstance(obj, Element):
            return cls(time=0, element=obj)
        if isinstance(obj, tuple):
            return cls(time=obj[0], element=obj[1])
        return obj


def _convert_abs_entries(
    entries: _typing.List[
        _typing.Union[Element, _typing.Tuple[float, Element], AbsoluteEntry]
    ]
) -> _typing.List[AbsoluteEntry]:
    return [AbsoluteEntry.from_tuple(obj) for obj in entries]


@_attrs.frozen
class Absolute(Element):
    """An absolute schedule element.

    The child elements are arranged in absolute time. The time of each child
    element is relative to the start of the absolute schedule. The duration of
    the absolute schedule is the maximum end time of the child elements.

    :param children: Child elements with absolute timing. Each item in the list
        can be either an :class:`Element` or ``(time, element)``. Default
        ``time`` is 0.
    :type children: list[Element | tuple[float, Element] | AbsoluteEntry]
    """

    TYPE_ID = 9

    children: _typing.List[AbsoluteEntry] = _attrs.field(
        converter=_convert_abs_entries, factory=list
    )

    def with_children(
        self, *children: _typing.Union[Element, _typing.Tuple[float, Element]]
    ) -> "Absolute":
        """Create a new absolute schedule with different children.

        :param children: The new children.
        :return: The new absolute schedule.
        """
        return _attrs.evolve(self, children=children)


class GridLengthUnit(_enum.Enum):
    """Unit of grid length."""

    SECOND = 0
    """Seconds."""
    AUTO = 1
    """Automatic."""
    STAR = 2
    """Fraction of remaining space."""


@_attrs.frozen
class GridLength(MsgObject):
    """Length of a grid column.

    :class:`GridLength` is used to specify the length of a grid column. The
    length can be specified in seconds, as a fraction of the remaining space,
    or automatically.
    """

    value: float
    unit: GridLengthUnit

    @classmethod
    def auto(cls) -> "GridLength":
        """Create an automatic grid length."""
        return cls(value=_math.nan, unit=GridLengthUnit.AUTO)

    @classmethod
    def star(cls, value: float) -> "GridLength":
        """Create a star grid length."""
        return cls(value=value, unit=GridLengthUnit.STAR)

    @classmethod
    def abs(cls, value: float) -> "GridLength":
        """Create an absolute grid length."""
        return cls(value=value, unit=GridLengthUnit.SECOND)

    @classmethod
    def parse(cls, value: _typing.Union[str, float]) -> "GridLength":
        """Create a grid length from a string or a float.

        The value can be one of the following formats:

        ``"10e-9"`` or 10e-9
            10 nanoseconds

        ``"*"``
            1 star

        ``"10*"``
            10 stars

        ``"auto"``
            Automatic

        :param value: The value to parse.
        """
        if isinstance(value, (float, int)):
            return cls.abs(value)
        if value.lower() == "auto":
            return cls.auto()
        if value.endswith("*"):
            return cls.star(float(value[:-1]))
        return cls.abs(float(value))


@_attrs.frozen
class GridEntry(MsgObject):
    """An entry in the grid schedule."""

    column: int
    span: int
    element: Element

    @classmethod
    def from_tuple(
        cls,
        obj: _typing.Union[
            Element,
            _typing.Tuple[int, Element],
            _typing.Tuple[int, int, Element],
            "GridEntry",
        ],
    ) -> "GridEntry":
        """Create a grid entry from a tuple.

        :param obj: The tuple to convert.
        """
        if isinstance(obj, Element):
            return cls(column=0, span=1, element=obj)
        if isinstance(obj, tuple):
            if len(obj) == 2:
                return cls(column=obj[0], span=1, element=obj[1])
            return cls(column=obj[0], span=obj[1], element=obj[2])
        return obj


def _convert_grid_entries(
    entries: _typing.List[
        _typing.Union[
            Element,
            _typing.Tuple[int, Element],
            _typing.Tuple[int, int, Element],
            GridEntry,
        ]
    ]
) -> _typing.List[GridEntry]:
    return [GridEntry.from_tuple(obj) for obj in entries]


def _convert_columns(
    columns: _typing.List[_typing.Union[GridLength, str, float]]
) -> _typing.List[GridLength]:
    return [
        GridLength.parse(column) if not isinstance(column, GridLength) else column
        for column in columns
    ]


@_attrs.frozen
class Grid(Element):
    """A grid schedule element.

    :param children: Child elements with column index and span. Each item in the
        list can be either :class:`Element`, ``(column, element)`` or
        ``(column, span, element)``. The default column is 0 and the default
        span is 1.
    :type children: list[Element | tuple[int, Element] | tuple[int, int, Element] | GridEntry]
    :param columns: Definitions of grid columns. The length of the columns can
        be specified as a :class:`GridLength`, a string, or a float. See
        :meth:`GridLength.parse` for details.
    :type columns: list[GridLength | str | float]
    """

    TYPE_ID = 10

    children: _typing.List[GridEntry] = _attrs.field(
        converter=_convert_grid_entries, factory=list
    )
    """Child elements with grid positioning."""
    columns: _typing.List[GridLength] = _attrs.field(
        converter=_convert_columns, factory=list
    )
    """Definitions of grid columns."""

    def with_children(
        self,
        *children: _typing.Union[
            Element,
            _typing.Tuple[int, Element],
            _typing.Tuple[int, int, Element],
        ],
    ) -> "Grid":
        """Create a new grid schedule with different children.

        :param children: The new children.
        :return: The new grid schedule.
        """
        return _attrs.evolve(self, children=children)


@_attrs.frozen
class Request(MsgObject):
    """A schedule request.

    :param channels: Information about the channels used in the schedule.
    :param shapes: Information about the shapes used in the schedule.
    :param schedule: The root element of the schedule.
    :param options: Various options for waveform generation.
    """

    channels: _typing.List[Channel] = _attrs.field(converter=list)
    shapes: _typing.List[Shape] = _attrs.field(converter=list)
    schedule: Element
    options: Options = _attrs.field(factory=Options)
