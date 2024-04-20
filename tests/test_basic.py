import numpy as np

import bosing


def test_basic():
    channels = [bosing.Channel("xy0", 100e6, 2e9, 100000)]
    shapes = [bosing.Hann()]
    schedule = bosing.Stack(duration=49.9e-6).with_children(bosing.Play(0, 0, 0.1, 100e-9))
    result = bosing.generate_waveforms(channels, shapes, schedule)
    assert "xy0" in result
    w = result["xy0"]
    assert len(w) == 100000
    assert w[0] == 0
    assert w[-1] == 0
    assert np.any(w != 0)


def test_mixing():
    shapes = [bosing.Hann()]
    schedule = bosing.Stack(duration=500e-9).with_children(
        bosing.Play(
            channel_id=0,
            amplitude=0.3,
            shape_id=0,
            width=100e-9,
            plateau=200e-9,
        ),
        bosing.Barrier(duration=10e-9),
    )
    freq = 30e6

    channels = [bosing.Channel("xy", freq, 2e9, 1000)]
    result = bosing.generate_waveforms(channels, shapes, schedule)
    w1 = result["xy"]

    channels = [bosing.Channel("xy", 0, 2e9, 1000)]
    result = bosing.generate_waveforms(channels, shapes, schedule)
    w2 = result["xy"]
    w2 = w2 * np.exp(1j * (2 * np.pi * freq * np.arange(1000) / 2e9))

    assert np.allclose(w1, w2)
