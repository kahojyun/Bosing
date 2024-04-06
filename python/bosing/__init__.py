"""Generates microwave pulses for superconducting quantum computing experiments.

This module wraps the ``Bosing`` C# library.

.. note::
    All phase values are in number of cycles. For example, a phase of 0.25 means
    pi/2 radians.

.. warning::
    This package is still in development and the API may change in the future.
"""

from ._utils import generate_waveforms
from .models import (
    Absolute,
    AbsoluteEntry,
    Alignment,
    Barrier,
    Biquad,
    Channel,
    Element,
    Grid,
    GridEntry,
    GridLength,
    Hann,
    Interp,
    IqCalibration,
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
    "IqCalibration",
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
