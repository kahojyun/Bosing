# ruff: noqa: PLR0913
from collections.abc import Iterable, Iterator, Mapping, Sequence
from typing import ClassVar, Literal, final

import numpy as np
import numpy.typing as npt
from matplotlib.axes import Axes
from typing_extensions import Self, TypeAlias

__all__ = [
    "Absolute",
    "AbsoluteEntry",
    "Alignment",
    "Barrier",
    "Channel",
    "Direction",
    "Element",
    "Grid",
    "GridEntry",
    "GridLength",
    "GridLengthUnit",
    "Hann",
    "Interp",
    "ItemKind",
    "OscState",
    "Play",
    "PlotArgs",
    "PlotItem",
    "Repeat",
    "SetFreq",
    "SetPhase",
    "Shape",
    "ShiftFreq",
    "ShiftPhase",
    "Stack",
    "SwapPhase",
    "generate_waveforms",
    "generate_waveforms_with_states",
]

_RichReprResult: TypeAlias = list[object]

@final
class Channel:
    def __new__(
        cls,
        base_freq: float,
        sample_rate: float,
        length: int,
        *,
        delay: float = ...,
        align_level: int = ...,
        iq_matrix: npt.ArrayLike | Sequence[Sequence[float]] | None = ...,
        offset: npt.ArrayLike | None = ...,
        iir: npt.ArrayLike | None = ...,
        fir: npt.ArrayLike | None = ...,
        filter_offset: bool = ...,
        is_real: bool = ...,
    ) -> Self: ...
    @property
    def base_freq(self) -> float: ...
    @property
    def sample_rate(self) -> float: ...
    @property
    def length(self) -> int: ...
    @property
    def delay(self) -> float: ...
    @property
    def align_level(self) -> int: ...
    @property
    def iq_matrix(self) -> npt.NDArray[np.float64] | None: ...
    @property
    def offset(self) -> npt.NDArray[np.float64] | None: ...
    @property
    def iir(self) -> npt.NDArray[np.float64] | None: ...
    @property
    def fir(self) -> npt.NDArray[np.float64] | None: ...
    @property
    def filter_offset(self) -> bool: ...
    @property
    def is_real(self) -> bool: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class Alignment:
    End: ClassVar[Alignment]
    Start: ClassVar[Alignment]
    Center: ClassVar[Alignment]
    Stretch: ClassVar[Alignment]
    @staticmethod
    def convert(
        obj: Literal["end", "start", "center", "stretch"] | Alignment,
    ) -> Alignment: ...

class Shape: ...

@final
class Hann(Shape):
    def __new__(cls) -> Self: ...

@final
class Interp(Shape):
    def __new__(
        cls,
        knots: Iterable[float],
        controls: Iterable[float],
        degree: float,
    ) -> Self: ...
    @property
    def knots(self) -> Sequence[float]: ...
    @property
    def controls(self) -> Sequence[float]: ...
    @property
    def degree(self) -> float: ...

class Element:
    @property
    def margin(self) -> tuple[float, float]: ...
    @property
    def alignment(self) -> Alignment: ...
    @property
    def phantom(self) -> bool: ...
    @property
    def duration(self) -> float | None: ...
    @property
    def max_duration(self) -> float: ...
    @property
    def min_duration(self) -> float: ...
    @property
    def label(self) -> str: ...
    def measure(self) -> float: ...
    def plot(
        self,
        ax: Axes | None = ...,
        *,
        channels: Sequence[str] | None = ...,
        max_depth: int = ...,
        show_label: bool = ...,
    ) -> Axes: ...

@final
class Play(Element):
    def __new__(
        cls,
        channel_id: str,
        shape_id: str | None,
        amplitude: float,
        width: float,
        *,
        plateau: float = ...,
        drag_coef: float = ...,
        frequency: float = ...,
        phase: float = ...,
        flexible: bool = ...,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    @property
    def channel_id(self) -> str: ...
    @property
    def shape_id(self) -> str | None: ...
    @property
    def amplitude(self) -> float: ...
    @property
    def width(self) -> float: ...
    @property
    def plateau(self) -> float: ...
    @property
    def drag_coef(self) -> float: ...
    @property
    def frequency(self) -> float: ...
    @property
    def phase(self) -> float: ...
    @property
    def flexible(self) -> bool: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class ShiftPhase(Element):
    def __new__(
        cls,
        channel_id: str,
        phase: float,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    @property
    def channel_id(self) -> str: ...
    @property
    def phase(self) -> float: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class SetPhase(Element):
    def __new__(
        cls,
        channel_id: str,
        phase: float,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    @property
    def channel_id(self) -> str: ...
    @property
    def phase(self) -> float: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class ShiftFreq(Element):
    def __new__(
        cls,
        channel_id: str,
        frequency: float,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    @property
    def channel_id(self) -> str: ...
    @property
    def frequency(self) -> float: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class SetFreq(Element):
    def __new__(
        cls,
        channel_id: str,
        frequency: float,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    @property
    def channel_id(self) -> str: ...
    @property
    def frequency(self) -> float: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class SwapPhase(Element):
    def __new__(
        cls,
        channel_id1: str,
        channel_id2: str,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    @property
    def channel_id1(self) -> str: ...
    @property
    def channel_id2(self) -> str: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class Barrier(Element):
    def __new__(
        cls,
        *channel_ids: str,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    @property
    def channel_ids(self) -> Sequence[str]: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class Repeat(Element):
    def __new__(
        cls,
        child: Element,
        count: int,
        spacing: float = ...,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    @property
    def child(self) -> Element: ...
    @property
    def count(self) -> int: ...
    @property
    def spacing(self) -> float: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class Direction:
    Forward: ClassVar[Direction]
    Backward: ClassVar[Direction]
    @staticmethod
    def convert(obj: Literal["forward", "backward"] | Direction) -> Direction: ...

@final
class Stack(Element):
    def __new__(
        cls,
        *children: Element,
        direction: Literal["forward", "backward"] | Direction = ...,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    def with_children(self, *children: Element) -> Stack: ...
    @property
    def direction(self) -> Direction: ...
    @property
    def children(self) -> Sequence[Element]: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

_AbsoluteEntryLike: TypeAlias = Element | tuple[float, Element] | AbsoluteEntry

@final
class AbsoluteEntry:
    def __new__(cls, time: float, element: Element) -> Self: ...
    @property
    def time(self) -> float: ...
    @property
    def element(self) -> Element: ...
    @staticmethod
    def convert(obj: _AbsoluteEntryLike) -> AbsoluteEntry: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class Absolute(Element):
    def __new__(
        cls,
        *children: _AbsoluteEntryLike,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    def with_children(self, *children: _AbsoluteEntryLike) -> Absolute: ...
    @property
    def children(self) -> Sequence[AbsoluteEntry]: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class GridLengthUnit:
    Seconds: ClassVar[GridLengthUnit]
    Auto: ClassVar[GridLengthUnit]
    Star: ClassVar[GridLengthUnit]

@final
class GridLength:
    def __new__(cls, value: float, unit: GridLengthUnit) -> Self: ...
    @property
    def value(self) -> float: ...
    @property
    def unit(self) -> GridLengthUnit: ...
    @staticmethod
    def auto() -> GridLength: ...
    @staticmethod
    def star(value: float) -> GridLength: ...
    @staticmethod
    def fixed(value: float) -> GridLength: ...
    @staticmethod
    def convert(obj: str | float | GridLength) -> GridLength: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

_GridEntryLike: TypeAlias = (
    Element | tuple[Element, int] | tuple[Element, int, int] | GridEntry
)

@final
class GridEntry:
    def __new__(cls, element: Element, column: int = ..., span: int = ...) -> Self: ...
    @property
    def column(self) -> int: ...
    @property
    def span(self) -> int: ...
    @property
    def element(self) -> Element: ...
    @staticmethod
    def convert(obj: _GridEntryLike) -> GridEntry: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class Grid(Element):
    def __new__(
        cls,
        *children: _GridEntryLike,
        columns: Sequence[str | float | GridLength] = ...,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"]
        | Alignment
        | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
        label: str | None = ...,
    ) -> Self: ...
    def with_children(
        self,
        *children: _GridEntryLike,
    ) -> Grid: ...
    @property
    def children(self) -> Sequence[GridEntry]: ...
    @property
    def columns(self) -> Sequence[GridLength]: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class OscState:
    def __new__(
        cls,
        base_freq: float,
        delta_freq: float,
        phase: float,
    ) -> Self: ...
    base_freq: float
    delta_freq: float
    phase: float
    def total_freq(self) -> float: ...
    def phase_at(self, time: float) -> float: ...
    def with_time_shift(self, time: float) -> Self: ...
    def __rich_repr__(self) -> _RichReprResult: ...  # undocumented

@final
class PlotArgs:
    @property
    def ax(self) -> Axes | None: ...
    @property
    def blocks(self) -> Iterator[PlotItem]: ...
    @property
    def channels(self) -> list[str]: ...
    @property
    def max_depth(self) -> int: ...
    @property
    def show_label(self) -> bool: ...

@final
class PlotItem:
    @property
    def channels(self) -> list[str]: ...
    @property
    def start(self) -> float: ...
    @property
    def span(self) -> float: ...
    @property
    def depth(self) -> int: ...
    @property
    def kind(self) -> ItemKind: ...
    @property
    def label(self) -> str | None: ...

@final
class ItemKind:
    Play: ClassVar[ItemKind]
    ShiftPhase: ClassVar[ItemKind]
    SetPhase: ClassVar[ItemKind]
    ShiftFreq: ClassVar[ItemKind]
    SetFreq: ClassVar[ItemKind]
    SwapPhase: ClassVar[ItemKind]
    Barrier: ClassVar[ItemKind]
    Repeat: ClassVar[ItemKind]
    Stack: ClassVar[ItemKind]
    Absolute: ClassVar[ItemKind]
    Grid: ClassVar[ItemKind]

def generate_waveforms(
    channels: Mapping[str, Channel],
    shapes: Mapping[str, Shape],
    schedule: Element,
    *,
    time_tolerance: float = ...,
    amp_tolerance: float = ...,
    allow_oversize: bool = ...,
    crosstalk: tuple[npt.ArrayLike, Sequence[str]] | None = ...,
) -> dict[str, npt.NDArray[np.float64]]: ...
def generate_waveforms_with_states(
    channels: Mapping[str, Channel],
    shapes: Mapping[str, Shape],
    schedule: Element,
    *,
    time_tolerance: float = ...,
    amp_tolerance: float = ...,
    allow_oversize: bool = ...,
    crosstalk: tuple[npt.ArrayLike, Sequence[str]] | None = ...,
    states: Mapping[str, OscState] | None = ...,
) -> tuple[dict[str, npt.NDArray[np.float64]], dict[str, OscState]]: ...
