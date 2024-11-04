r"""Generates microwave pulses for superconducting quantum computing experiments.

.. caution::

    The unit of phase is number of cycles, not radians. For example, a phase
    of :math:`0.5` means a phase shift of :math:`\pi` radians.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, final

import numpy as np
from typing_extensions import override

from . import _bosing, _repr
from ._bosing import (
    Absolute,
    AbsoluteEntry,
    Alignment,
    Barrier,
    Direction,
    Element,
    Grid,
    GridEntry,
    GridLength,
    GridLengthUnit,
    Hann,
    Interp,
    Play,
    Repeat,
    SetFreq,
    SetPhase,
    Shape,
    ShiftFreq,
    ShiftPhase,
    Stack,
    SwapPhase,
)

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

    import numpy.typing as npt

    from ._repr import TupleRichReprResult


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
    "OscState",
    "Play",
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


@final
class Channel:
    r"""Channel configuration.

    :attr:`align_level` controls the time axis alignment granularity. With sampling
    interval :math:`\Delta t` and :attr:`align_level` :math:`n`, start of pulse is
    aligned to the nearest multiple of :math:`2^n \Delta t`.

    Each channel can be either real or complex. If the channel is complex, the
    filter will be applied to both I and Q components.

    .. caution::

        Crosstalk matrix will not be applied to offset.
    """

    def __init__(  # noqa: PLR0913
        self,
        base_freq: float,
        sample_rate: float,
        length: int,
        *,
        delay: float = 0.0,
        align_level: int = -10,
        iq_matrix: npt.ArrayLike | Sequence[Sequence[float]] | None = None,
        offset: npt.ArrayLike | None = None,
        iir: npt.ArrayLike | None = None,
        fir: npt.ArrayLike | None = None,
        filter_offset: bool = False,
        is_real: bool = False,
    ) -> None:
        """Initializes the channel.

        Args:
            base_freq: Base frequency of the channel.
            sample_rate: Sample rate of the channel.
            length: Length of the waveform.
            delay: Delay of the channel. Defaults to ``0.0``.
            align_level: Time axis alignment granularity. Defaults to ``-10``.
            iq_matrix: IQ matrix of the channel. Defaults to ``None``.
            offset: Offsets of the channel. The length of the sequence should
                be 2 if the channel is complex, or 1 if the channel is real.
                Defaults to ``None``.
            iir: IIR filter of the channel. The format of the array is ``[[b0,
                b1, b2, a0, a1, a2], ...]``, which is the same as `sos` parameter
                of :func:`scipy.signal.sosfilt`. Defaults to ``None``.
            fir: FIR filter of the channel. Defaults to ``None``.
            filter_offset: Whether to apply filter to the offset. Defaults to
                ``False``.
            is_real: Whether the channel is real. Defaults to ``False``.
        """
        if is_real and iq_matrix is not None:
            msg = "iq_matrix should be None for real channel"
            raise ValueError(msg)
        iq_matrix = (
            None if iq_matrix is None else np.asarray(iq_matrix, dtype=np.float64)
        )
        if iq_matrix is not None and iq_matrix.shape != (2, 2):
            msg = "iq_matrix should have shape (2, 2)"
            raise ValueError(msg)

        offset = None if offset is None else np.asarray(offset, dtype=np.float64)
        if offset is not None:
            if is_real and offset.shape != (1,):
                msg = "offset should have length 1 for real channel"
                raise ValueError(msg)
            if not is_real and offset.shape != (2,):
                msg = "offset should have length 2 for complex channel"
                raise ValueError(msg)

        iir = None if iir is None else np.asarray(iir, dtype=np.float64)
        if iir is not None and (iir.ndim != 2 or iir.shape[1] != 6):  # noqa: PLR2004
            msg = "iir should have shape (N, 6)"
            raise ValueError(msg)

        fir = None if fir is None else np.asarray(fir, dtype=np.float64)
        if fir is not None and fir.ndim != 1:
            msg = "fir should be a 1D array"
            raise ValueError(msg)

        self._base_freq = base_freq
        self._sample_rate = sample_rate
        self._length = length
        self._delay = delay
        self._align_level = align_level
        self._iq_matrix = iq_matrix
        self._offset = offset
        self._iir = iir
        self._fir = fir
        self._filter_offset = filter_offset
        self._is_real = is_real

    @property
    def base_freq(self) -> float:
        """Base frequency of the channel."""
        return self._base_freq

    @property
    def sample_rate(self) -> float:
        """Sample rate of the channel."""
        return self._sample_rate

    @property
    def length(self) -> int:
        """Number of samples in the waveform."""
        return self._length

    @property
    def delay(self) -> float:
        """Delay of the channel."""
        return self._delay

    @property
    def align_level(self) -> int:
        r"""Time axis alignment granularity.

        Start of pulse is aligned to the nearest multiple of :math:`2^n \Delta t`.
        """
        return self._align_level

    @property
    def iq_matrix(self) -> npt.NDArray[np.float64] | None:
        """IQ calibration matrix of the channel.

        The shape of the matrix is ``(2, 2)``.
        """
        return self._iq_matrix

    @property
    def offset(self) -> npt.NDArray[np.float64] | None:
        """Offsets of each sub-channel.

        The length of the array should be 2 if the channel is complex, or 1 if
        the channel is real.
        """
        return self._offset

    @property
    def iir(self) -> npt.NDArray[np.float64] | None:
        """IIR filter to be applied to the channel.

        The shape of the array is ``(N, 6)``, the same format as `scipy.signal.sosfilt`.
        """
        return self._iir

    @property
    def fir(self) -> npt.NDArray[np.float64] | None:
        """FIR filter to be applied to the channel.

        The shape of the array is ``(M,)``.
        """
        return self._fir

    @property
    def filter_offset(self) -> bool:
        """Whether to apply filter to the offset."""
        return self._filter_offset

    @property
    def is_real(self) -> bool:
        """Whether the channel is real.

        Real channels will only have the real part of the waveform generated.
        """
        return self._is_real

    def __rich_repr__(self) -> TupleRichReprResult:
        """Rich pretty-printing."""
        yield (self._base_freq,)
        yield (self._sample_rate,)
        yield (self._length,)
        yield "delay", self._delay, 0.0
        yield "align_level", self._align_level, -10
        if self._iq_matrix is not None:
            yield "iq_matrix", self._iq_matrix
        if self._offset is not None:
            yield "offset", self._offset
        if self._iir is not None:
            yield "iir", self._iir
        if self._fir is not None:
            yield "fir", self._fir
        yield "filter_offset", self._filter_offset, False
        yield "is_real", self._is_real, False

    @override
    def __repr__(self) -> str:
        return _repr.repr_from_rich(self)


@final
class OscState:
    """State of a channel oscillator."""

    def __init__(self, base_freq: float, delta_freq: float, phase: float) -> None:
        """Initializes the oscillator state.

        Args:
            base_freq: Base frequency of the oscillator.
            delta_freq: Frequency shift of the oscillator.
            phase: Phase of the oscillator in **cycles**.
        """
        self.inner = _bosing.OscState(
            base_freq=base_freq, delta_freq=delta_freq, phase=phase
        )

    @property
    def base_freq(self) -> float:
        """Base frequency of the oscillator."""
        return self.inner.base_freq

    @base_freq.setter
    def base_freq(self, value: float) -> None:
        self.inner.base_freq = value

    @property
    def delta_freq(self) -> float:
        """Frequency shift of the oscillator."""
        return self.inner.delta_freq

    @delta_freq.setter
    def delta_freq(self, value: float) -> None:
        self.inner.delta_freq = value

    @property
    def phase(self) -> float:
        """Phase of the oscillator in **cycles**."""
        return self.inner.phase

    @phase.setter
    def phase(self, value: float) -> None:
        self.inner.phase = value

    def total_freq(self) -> float:
        """Calculate the total frequency of the oscillator.

        Returns:
            Total frequency of the oscillator.
        """
        return self.inner.total_freq()

    def phase_at(self, time: float) -> float:
        """Calculate the phase of the oscillator at a given time.

        Args:
            time: Time.

        Returns:
            Phase of the oscillator in **cycles**.
        """
        return self.inner.phase_at(time)

    def with_time_shift(self, time: float) -> OscState:
        """Get a new state with a time shift.

        Args:
            time: Time shift.

        Returns:
            The new state.
        """
        return _wrap_osc_state(self.inner.with_time_shift(time))

    def __rich_repr__(self) -> TupleRichReprResult:
        """Rich pretty-printing."""
        yield "base_freq", self.base_freq
        yield "delta_freq", self.delta_freq
        yield "phase", self.phase

    @override
    def __repr__(self) -> str:
        return _repr.repr_from_rich(self)


def _wrap_osc_state(obj: _bosing.OscState) -> OscState:
    ret = OscState.__new__(OscState)
    ret.inner = obj
    return ret


def generate_waveforms(  # noqa: PLR0913
    channels: Mapping[str, Channel],
    shapes: Mapping[str, Shape],
    schedule: Element,
    *,
    time_tolerance: float = 1e-12,
    amp_tolerance: float = 0.1 / 2**16,
    allow_oversize: bool = False,
    crosstalk: tuple[npt.ArrayLike, Sequence[str]] | None = None,
) -> dict[str, npt.NDArray[np.float64]]:
    r"""Generate waveforms from a schedule.

    .. caution::

        Crosstalk matrix will not be applied to offset of the channels.

    Args:
        channels: Information of the channels.
        shapes: Shapes used in the schedule.
        schedule: Root element of the schedule.
        time_tolerance: Tolerance for time comparison. Default is ``1e-12``.
        amp_tolerance: Tolerance for amplitude comparison. Default is ``0.1 /
            2**16``.
        allow_oversize: Allow elements to occupy a longer duration than
            available. Default is ``False``.
        crosstalk: Crosstalk matrix with corresponding channel ids. Default is
            ``None``.

    Returns:
        Waveforms of the channels. The key is the channel name and the value is
        the waveform. The shape of the waveform is ``(n, length)``, where ``n``
        is 2 for complex waveform and 1 for real waveform.

    Raises:
        ValueError: If some input is invalid.
        TypeError: If some input has an invalid type.
        RuntimeError: If waveform generation fails.

    Example:
        .. code-block:: python

            from bosing import Barrier, Channel, Hann, Play, Stack, generate_waveforms
            channels = {"xy": Channel(30e6, 2e9, 1000)}
            shapes = {"hann": Hann()}
            schedule = Stack(duration=500e-9).with_children(
                Play(
                    channel_id="xy",
                    shape_id="hann",
                    amplitude=0.3,
                    width=100e-9,
                    plateau=200e-9,
                ),
                Barrier(duration=10e-9),
            )
            result = generate_waveforms(channels, shapes, schedule)
    """
    return _bosing.generate_waveforms(
        channels=channels,
        shapes=shapes,
        schedule=schedule,
        time_tolerance=time_tolerance,
        amp_tolerance=amp_tolerance,
        allow_oversize=allow_oversize,
        crosstalk=crosstalk,
    )


def generate_waveforms_with_states(  # noqa: PLR0913
    channels: Mapping[str, Channel],
    shapes: Mapping[str, Shape],
    schedule: Element,
    *,
    time_tolerance: float = 1e-12,
    amp_tolerance: float = 0.1 / 2**16,
    allow_oversize: bool = False,
    crosstalk: tuple[npt.ArrayLike, Sequence[str]] | None = None,
    states: Mapping[str, OscState] | None = None,
) -> tuple[dict[str, npt.NDArray[np.float64]], dict[str, OscState]]:
    r"""Generate waveforms from a schedule with initial states.

    .. caution::

        Crosstalk matrix will not be applied to offset of the channels.

    Args:
        channels: Information of the channels.
        shapes: Shapes used in the schedule.
        schedule: Root element of the schedule.
        time_tolerance: Tolerance for time comparison. Default is ``1e-12``.
        amp_tolerance: Tolerance for amplitude comparison. Default is ``0.1 /
            2**16``.
        allow_oversize: Allow elements to occupy a longer duration than
            available. Default is ``False``.
        crosstalk: Crosstalk matrix with corresponding channel ids. Default is
            ``None``.
        states: Initial states of the channels.

    Returns:
        A tuple of waveforms and final states.

        Waveforms part is a dictionary mapping channel names to waveforms. The
        shape of the waveform is ``(n, length)``, where ``n`` is 2 for complex
        waveform and 1 for real waveform.

        States part is a dictionary mapping channel names to the final states.

    Raises:
        ValueError: If some input is invalid.
        TypeError: If some input has an invalid type.
        RuntimeError: If waveform generation fails.
    """
    inner_states = (
        {k: v.inner for k, v in states.items()} if states is not None else None
    )
    waveforms, new_states = _bosing.generate_waveforms_with_states(
        channels=channels,
        shapes=shapes,
        schedule=schedule,
        time_tolerance=time_tolerance,
        amp_tolerance=amp_tolerance,
        allow_oversize=allow_oversize,
        crosstalk=crosstalk,
        states=inner_states,
    )
    new_states = {k: _wrap_osc_state(v) for k, v in new_states.items()}
    return waveforms, new_states
