import typing

import numpy as np

from ._native import copy_waveform, free_waveform, run
from .models import Request


def generate_waveforms(
    request: Request,
) -> typing.Dict[str, typing.Tuple[np.ndarray, np.ndarray]]:
    """Generate waveforms for the given request.

    :param request: The request to generate waveforms for.
    :return: A dictionary of waveforms, where the key is the channel name and
        the value is a tuple of two numpy arrays representing the I and Q
        components of the waveform.
    """
    msg = request.packb()
    handle = run(msg)
    try:
        waveforms = {}
        for ch in request.channels:
            waveforms[ch.name] = copy_waveform(handle, ch.name, ch.length)
        return waveforms
    finally:
        free_waveform(handle)
