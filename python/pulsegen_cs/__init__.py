"""Generates microwave pulses for superconducting quantum computing experiments.

This module wraps the ``Qynit.PulseGen`` C# library.

.. note::
    All phase values are in number of cycles. For example, a phase of 0.25 means
    pi/2 radians.

.. warning::
    This package is still in development and the API may change in the future.
"""

from ._utils import generate_waveforms
from .models import (
    Absolute,
    Alignment,
    Barrier,
    Biquad,
    ChannelInfo,
    Grid,
    HannShape,
    InterpolatedShape,
    IqCalibration,
    Options,
    Play,
    Repeat,
    Request,
    SetFrequency,
    SetPhase,
    ShiftFrequency,
    ShiftPhase,
    Stack,
    SwapPhase,
    TriangleShape,
)

__all__ = [
    "Absolute",
    "Alignment",
    "Barrier",
    "Biquad",
    "ChannelInfo",
    "Grid",
    "HannShape",
    "InterpolatedShape",
    "IqCalibration",
    "Options",
    "Play",
    "Repeat",
    "Request",
    "SetFrequency",
    "SetPhase",
    "ShiftFrequency",
    "ShiftPhase",
    "Stack",
    "SwapPhase",
    "TriangleShape",
    "generate_waveforms",
]
