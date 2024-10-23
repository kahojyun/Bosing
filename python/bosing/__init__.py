r"""Generates microwave pulses for superconducting quantum computing experiments.

.. caution::

    The unit of phase is number of cycles, not radians. For example, a phase
    of :math:`0.5` means a phase shift of :math:`\pi` radians.
"""

from ._bosing import (
    Absolute,
    AbsoluteEntry,
    Alignment,
    Barrier,
    Channel,
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
