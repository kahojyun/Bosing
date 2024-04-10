from collections.abc import Iterable, Sequence
from typing import ClassVar, Literal, final

import numpy as np

@final
class Channel:
    def __init__(
        self,
        name: str,
        base_freq: float,
        sample_rate: float,
        length: int,
        *,
        delay: float = ...,
        align_level: int = ...,
    ) -> None: ...
    @property
    def name(self) -> str: ...
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
class Options:
    def __init__(
        self,
        *,
        time_tolerance: float = ...,
        amp_tolerance: float = ...,
        phase_tolerance: float = ...,
        allow_oversize: bool = ...,
    ) -> None: ...
    @property
    def time_tolerance(self) -> float: ...
    @property
    def amp_tolerance(self) -> float: ...
    @property
    def phase_tolerance(self) -> float: ...
    @property
    def allow_oversize(self) -> bool: ...

@final
class Alignment:
    End: ClassVar[Alignment]
    Start: ClassVar[Alignment]
    Center: ClassVar[Alignment]
    Stretch: ClassVar[Alignment]
    @classmethod
    def from_str(cls, s: Literal["end", "start", "center", "stretch"]) -> Alignment: ...

class Shape: ...

@final
class Hann(Shape):
    def __init__(self) -> None: ...

@final
class Interp(Shape):
    def __init__(self, xs: Iterable[float], ys: Iterable[float]) -> None: ...
    @property
    def xs(self) -> Sequence[float]: ...
    @property
    def ys(self) -> Sequence[float]: ...

class Element:
    @property
    def margin(self) -> tuple[float, float]: ...
    @property
    def alignment(self) -> Alignment: ...
    @property
    def visibility(self) -> bool: ...
    @property
    def duration(self) -> float | None: ...
    @property
    def max_duration(self) -> float: ...
    @property
    def min_duration(self) -> float: ...

@final
class Play(Element):
    def __init__(
        self,
        channel_id: int,
        amplitude: float,
        shape_id: int | None,
        width: float,
        *,
        plateau: float = ...,
        drag_coef: float = ...,
        frequency: float = ...,
        phase: float = ...,
        flexible: bool = ...,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    @property
    def channel_id(self) -> int: ...
    @property
    def amplitude(self) -> float: ...
    @property
    def shape_id(self) -> int | None: ...
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
    def __init__(
        self,
        channel_id: int,
        phase: float,
        *,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    @property
    def channel_id(self) -> int: ...
    @property
    def phase(self) -> float: ...

@final
class SetPhase(Element):
    def __init__(
        self,
        channel_id: int,
        phase: float,
        *,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    @property
    def channel_id(self) -> int: ...
    @property
    def phase(self) -> float: ...

@final
class ShiftFreq(Element):
    def __init__(
        self,
        channel_id: int,
        frequency: float,
        *,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    @property
    def channel_id(self) -> int: ...
    @property
    def frequency(self) -> float: ...

@final
class SetFreq(Element):
    def __init__(
        self,
        channel_id: int,
        frequency: float,
        *,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    @property
    def channel_id(self) -> int: ...
    @property
    def frequency(self) -> float: ...

@final
class SwapPhase(Element):
    def __init__(
        self,
        channel_id1: int,
        channel_id2: int,
        *,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    @property
    def channel_id1(self) -> int: ...
    @property
    def channel_id2(self) -> int: ...

@final
class Barrier(Element):
    def __init__(
        self,
        channel_ids: Iterable[int] = ...,
        *,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    @property
    def channel_ids(self) -> Sequence[int]: ...

@final
class Repeat(Element):
    def __init__(
        self,
        child: Element,
        count: int,
        spacing: float = ...,
        *,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
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
    @classmethod
    def from_str(cls, s: Literal["forward", "backward"]) -> Direction: ...

@final
class Stack(Element):
    def __init__(
        self,
        children: Iterable[Element] = ...,
        *,
        direction: Literal["forward", "backward"] | Direction = ...,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    def with_children(self, *children: Element) -> Stack: ...
    @property
    def direction(self) -> Direction: ...
    @property
    def children(self) -> Sequence[Element]: ...

@final
class AbsoluteEntry:
    def __init__(self, time: float, element: Element) -> None: ...
    @property
    def time(self) -> float: ...
    @property
    def element(self) -> Element: ...
    @classmethod
    def convert(
        cls, obj: Element | tuple[float, Element] | AbsoluteEntry
    ) -> AbsoluteEntry: ...

@final
class Absolute(Element):
    def __init__(
        self,
        children: Iterable[Element | tuple[float, Element] | AbsoluteEntry] = ...,
        *,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    def with_children(
        self, *children: Element | tuple[float, Element] | AbsoluteEntry
    ) -> Absolute: ...
    @property
    def children(self) -> Sequence[AbsoluteEntry]: ...

@final
class GridLengthUnit:
    Seconds: ClassVar[GridLengthUnit]
    Auto: ClassVar[GridLengthUnit]
    Star: ClassVar[GridLengthUnit]

@final
class GridLength:
    def __init__(self, value: float, unit: GridLengthUnit) -> None: ...
    @property
    def value(self) -> float: ...
    @property
    def unit(self) -> GridLengthUnit: ...
    @classmethod
    def auto(cls) -> GridLength: ...
    @classmethod
    def star(cls, value: float) -> GridLength: ...
    @classmethod
    def fixed(cls, value: float) -> GridLength: ...
    @classmethod
    def parse(cls, s: str | float) -> GridLength: ...

@final
class GridEntry:
    def __init__(self, column: int, span: int, element: Element) -> None: ...
    @property
    def column(self) -> int: ...
    @property
    def span(self) -> int: ...
    @property
    def element(self) -> Element: ...
    @classmethod
    def convert(
        cls, obj: Element | tuple[int, Element] | tuple[int, int, Element] | GridEntry
    ) -> GridEntry: ...

@final
class Grid(Element):
    def __init__(
        self,
        children: Iterable[
            Element | tuple[int, Element] | tuple[int, int, Element] | GridEntry
        ] = ...,
        columns: Sequence[GridLength] = ...,
        *,
        margin: float | tuple[float, float] = ...,
        alignment: Literal["end", "start", "center", "stretch"] | Alignment = ...,
        visibility: bool = ...,
        duration: float | None = ...,
        max_duration: float = ...,
        min_duration: float = ...,
    ) -> None: ...
    def with_children(
        self,
        *children: Element | tuple[int, Element] | tuple[int, int, Element] | GridEntry,
    ) -> Grid: ...
    @property
    def children(self) -> Sequence[GridEntry]: ...
    @property
    def columns(self) -> Sequence[GridLength]: ...

def generate_waveforms(
    channels: Iterable[Channel],
    shapes: Iterable[Shape],
    schedule: Element,
    options: Options = ...,
) -> dict[str, tuple[np.ndarray, np.ndarray]]: ...
