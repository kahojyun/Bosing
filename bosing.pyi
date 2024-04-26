from collections.abc import Iterable, Mapping, Sequence
from typing import ClassVar, Literal, Self, TypeAlias, final

import numpy as np
import numpy.typing as npt

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

@final
class Alignment:
    End: ClassVar[Alignment]
    Start: ClassVar[Alignment]
    Center: ClassVar[Alignment]
    Stretch: ClassVar[Alignment]
    @staticmethod
    def convert(obj: Literal["end", "start", "center", "stretch"] | Alignment) -> Alignment: ...

class Shape: ...

@final
class Hann(Shape):
    def __new__(cls) -> Self: ...

@final
class Interp(Shape):
    def __new__(cls, knots: Iterable[float], controls: Iterable[float], degree: float) -> Self: ...
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
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
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

@final
class ShiftPhase(Element):
    def __new__(
        cls,
        channel_id: str,
        phase: float,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    @property
    def channel_id(self) -> str: ...
    @property
    def phase(self) -> float: ...

@final
class SetPhase(Element):
    def __new__(
        cls,
        channel_id: str,
        phase: float,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    @property
    def channel_id(self) -> str: ...
    @property
    def phase(self) -> float: ...

@final
class ShiftFreq(Element):
    def __new__(
        cls,
        channel_id: str,
        frequency: float,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    @property
    def channel_id(self) -> str: ...
    @property
    def frequency(self) -> float: ...

@final
class SetFreq(Element):
    def __new__(
        cls,
        channel_id: str,
        frequency: float,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    @property
    def channel_id(self) -> str: ...
    @property
    def frequency(self) -> float: ...

@final
class SwapPhase(Element):
    def __new__(
        cls,
        channel_id1: str,
        channel_id2: str,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    @property
    def channel_id1(self) -> str: ...
    @property
    def channel_id2(self) -> str: ...

@final
class Barrier(Element):
    def __new__(
        cls,
        *channel_ids: str,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    @property
    def channel_ids(self) -> Sequence[str]: ...

@final
class Repeat(Element):
    def __new__(
        cls,
        child: Element,
        count: int,
        spacing: float = ...,
        *,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    @property
    def child(self) -> Element: ...
    @property
    def count(self) -> int: ...
    @property
    def spacing(self) -> float: ...

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
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    def with_children(self, *children: Element) -> Stack: ...
    @property
    def direction(self) -> Direction: ...
    @property
    def children(self) -> Sequence[Element]: ...

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

@final
class Absolute(Element):
    def __new__(
        cls,
        *children: _AbsoluteEntryLike,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    def with_children(self, *children: _AbsoluteEntryLike) -> Absolute: ...
    @property
    def children(self) -> Sequence[AbsoluteEntry]: ...

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

_GridEntryLike: TypeAlias = Element | tuple[Element, int] | tuple[Element, int, int] | GridEntry

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

@final
class Grid(Element):
    def __new__(
        cls,
        *children: _GridEntryLike,
        columns: Sequence[str | float | GridLength] = ...,
        margin: float | tuple[float, float] | None = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment | None = ...,
        phantom: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> Self: ...
    def with_children(
        self,
        *children: _GridEntryLike,
    ) -> Grid: ...
    @property
    def children(self) -> Sequence[GridEntry]: ...
    @property
    def columns(self) -> Sequence[GridLength]: ...

def generate_waveforms(
    channels: Mapping[str, Channel],
    shapes: Mapping[str, Shape],
    schedule: Element,
    *,
    time_tolerance: float = ...,
    amp_tolerance: float = ...,
    allow_oversize: bool = ...,
    crosstalk: tuple[npt.ArrayLike, Sequence[str]] | None = ...,
) -> dict[str, np.ndarray]: ...
