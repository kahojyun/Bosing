"""This module provides a client for the pulsegen service.

The client can be used to send requests to the pulsegen server and receive the
result. There are two clients available: a synchronous client and an asynchronous
client.

.. note::
    All phase values are in number of cycles. For example, a phase of 0.25 means
    pi/2 radians.

.. warning::
    This package is still in development and the API may change in the future.
"""

from ._native import generate_waveforms
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
    "ChannelInfo",
    "Biquad",
    "IqCalibration",
    "Options",
    "Absolute",
    "Alignment",
    "Barrier",
    "Grid",
    "Play",
    "Repeat",
    "Request",
    "SetFrequency",
    "SetPhase",
    "ShiftFrequency",
    "ShiftPhase",
    "Stack",
    "SwapPhase",
    "HannShape",
    "InterpolatedShape",
    "TriangleShape",
    "generate_waveforms",
]
