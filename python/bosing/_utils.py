from __future__ import annotations

from collections.abc import Iterable  # noqa: TCH003

import numpy as np  # noqa: TCH002

from bosing._native import copy_waveform, free_waveform, run
from bosing.models import Channel, Element, Options, Request, Shape


def generate_waveforms(
    channels: Iterable[Channel],
    shapes: Iterable[Shape],
    schedule: Element,
    options: Options | None = None,
) -> dict[str, tuple[np.ndarray, np.ndarray]]:
    """Generate waveforms.

    :param channels: Information about the channels used in the schedule.
    :param shapes: Information about the shapes used in the schedule.
    :param schedule: The root element of the schedule.
    :param options: Various options for the waveform generation.
    :return: A dictionary of waveforms, where the key is the channel name and
        the value is a tuple of two numpy arrays representing the I and Q
        components of the waveform.
    :raises RuntimeError: If the waveform generation fails.
    """
    request = Request(
        channels=channels,
        shapes=shapes,
        schedule=schedule,
        options=options or Options(),
    )
    msg = request.packb()
    handle = run(msg)
    try:
        waveforms = {}
        for ch in request.channels:
            waveforms[ch.name] = copy_waveform(handle, ch.name, ch.length)
        return waveforms
    finally:
        free_waveform(handle)
