"""Generates microwave pulses for superconducting quantum computing experiments.

.. caution::

    All phase values are in number of cycles. For example, a phase of
    :math:`0.25` means :math:`\\pi/2` radians.
"""

from bosing._utils import generate_waveforms
from bosing.models import (
    Absolute,
    AbsoluteEntry,  # noqa: F401
    Alignment,
    Barrier,
    Biquad,
    Channel,
    Element,  # noqa: F401
    Grid,
    GridEntry,  # noqa: F401
    GridLength,  # noqa: F401
    Hann,
    Interp,
    IqCali,
    Options,
    Play,
    Repeat,
    SetFreq,
    SetPhase,
    ShiftFreq,
    ShiftPhase,
    Stack,
    SwapPhase,
)

__all__ = [
    "Absolute",
    "Alignment",
    "Barrier",
    "Biquad",
    "Channel",
    "Grid",
    "Hann",
    "Interp",
    "IqCali",
    "Options",
    "Play",
    "Repeat",
    "SetFreq",
    "SetPhase",
    "ShiftFreq",
    "ShiftPhase",
    "Stack",
    "SwapPhase",
    "generate_waveforms",
]
