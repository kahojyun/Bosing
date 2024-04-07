import numpy as np

import bosing


def test_basic():
    channels = [bosing.Channel("xy0", 100e6, 2e9, 100000)]
    shapes = [bosing.Hann()]
    schedule = bosing.Stack(duration=49.9e-6).with_children(
        bosing.Play(0, 0.1, 0, 100e-9)
    )
    result = bosing.generate_waveforms(channels, shapes, schedule)
    assert "xy0" in result
    i, q = result["xy0"]
    assert len(i) == len(q)
    assert len(i) == 100000
    assert i[0] == 0
    assert i[-1] == 0
    assert np.any(i != 0)
    assert q[0] == 0
    assert q[-1] == 0
    assert np.any(q != 0)


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
    i1, q1 = result["xy"]
    w1 = i1 + 1j * q1

    channels = [bosing.Channel("xy", 0, 2e9, 1000)]
    result = bosing.generate_waveforms(channels, shapes, schedule)
    i2, q2 = result["xy"]
    w2 = i2 + 1j * q2
    w2 = w2 * np.exp(1j * (2 * np.pi * freq * np.arange(1000) / 2e9))

    assert np.allclose(w1, w2)
