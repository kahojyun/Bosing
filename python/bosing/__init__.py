r"""Generates microwave pulses for superconducting quantum computing experiments.

.. caution::

    The unit of phase is number of cycles, not radians. For example, a phase
    of :math:`0.5` means a phase shift of :math:`\pi` radians.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, final

import numpy as np

from . import _bosing
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
    OscState,
    Play,
    Repeat,
    SetFreq,
    SetPhase,
    Shape,
    ShiftFreq,
    ShiftPhase,
    Stack,
    SwapPhase,
    generate_waveforms,
    generate_waveforms_with_states,
)

if TYPE_CHECKING:
    from collections.abc import Sequence

    import numpy.typing as npt

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

    `align_level` is the time axis alignment granularity. With sampling interval
    :math:`\Delta t` and `align_level` :math:`n`, start of pulse is aligned to
    the nearest multiple of :math:`2^n \Delta t`.

    Each channel can be either real or complex. If the channel is complex, the
    filter will be applied to both I and Q components. If the channel is real,
    `iq_matrix` will be ignored.

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
            base_freq (float): Base frequency of the channel.
            sample_rate (float): Sample rate of the channel.
            length (int): Length of the waveform.
            delay (float): Delay of the channel. Defaults to 0.0.
            align_level (int): Time axis alignment granularity. Defaults to -10.
            iq_matrix (array_like[2, 2] | None): IQ matrix of the channel. Defaults
                to ``None``.
            offset (Sequence[float] | None): Offsets of the channel. The length of the
                sequence should be 2 if the channel is complex, or 1 if the channel is
                real. Defaults to ``None``.
            iir (array_like[N, 6] | None): IIR filter of the channel. The format of
                the array is ``[[b0, b1, b2, a0, a1, a2], ...]``, which is the same
                as `sos` parameter of :func:`scipy.signal.sosfilt`. Defaults to
                ``None``.
            fir (array_like[M] | None): FIR filter of the channel. Defaults to None.
            filter_offset (bool): Whether to apply filter to the offset. Defaults to
                ``False``.
            is_real (bool): Whether the channel is real. Defaults to ``False``.
        """
        self._inner = _bosing.Channel(
            base_freq,
            sample_rate,
            length,
            delay=delay,
            align_level=align_level,
            iq_matrix=iq_matrix,
            offset=offset,
            iir=iir,
            fir=fir,
            filter_offset=filter_offset,
            is_real=is_real,
        )

    @property
    def base_freq(self) -> float:
        """Base frequency of the channel."""
        return self._inner.base_freq

    @property
    def sample_rate(self) -> float:
        """Sample rate of the channel."""
        return self._inner.sample_rate

    @property
    def length(self) -> int:
        """Number of samples in the waveform."""
        return self._inner.length

    @property
    def delay(self) -> float:
        """Delay of the channel."""
        return self._inner.delay

    @property
    def align_level(self) -> int:
        r"""Time axis alignment granularity.

        Start of pulse is aligned to the nearest multiple of :math:`2^n \Delta t`.
        """
        return self._inner.align_level

    @property
    def iq_matrix(self) -> npt.NDArray[np.float64] | None:
        """IQ calibration matrix of the channel.

        The shape of the matrix is ``(2, 2)``.
        """
        return self._inner.iq_matrix

    @property
    def offset(self) -> npt.NDArray[np.float64] | None:
        """Offsets of each sub-channel.

        The length of the array should be 2 if the channel is complex, or 1 if
        the channel is real.
        """
        return self._inner.offset

    @property
    def iir(self) -> npt.NDArray[np.float64] | None:
        """IIR filter to be applied to the channel.

        The shape of the array is ``(N, 6)``, the same format as `scipy.signal.sosfilt`.
        """
        return self._inner.iir

    @property
    def fir(self) -> npt.NDArray[np.float64] | None:
        """FIR filter to be applied to the channel.

        The shape of the array is ``(M,)``.
        """
        return self._inner.fir

    @property
    def filter_offset(self) -> bool:
        """Whether to apply filter to the offset."""
        return self._inner.filter_offset

    @property
    def is_real(self) -> bool:
        """Whether the channel is real.

        Real channels will only have the real part of the waveform generated.
        """
        return self._inner.is_real
