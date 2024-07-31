import numpy as np

import bosing


def test_basic():
    channels = {"xy0": bosing.Channel(100e6, 2e9, 100000)}
    shapes = {"hann": bosing.Hann()}
    schedule = bosing.Stack(duration=49.9e-6).with_children(bosing.Play("xy0", "hann", 0.1, 100e-9))
    result = bosing.generate_waveforms(channels, shapes, schedule)
    assert "xy0" in result
    w = result["xy0"]
    assert w.shape == (2, 100000)
    w = w[0] + 1j * w[1]
    assert w[0] == 0
    assert w[-1] == 0
    assert np.any(w != 0)


def test_mixing():
    shapes = {"hann": bosing.Hann()}
    schedule = bosing.Stack(duration=500e-9).with_children(
        bosing.Play(
            channel_id="xy",
            shape_id="hann",
            amplitude=0.3,
            width=100e-9,
            plateau=200e-9,
        ),
        bosing.Barrier(duration=10e-9),
    )
    freq = 30e6

    channels = {"xy": bosing.Channel(freq, 2e9, 1000)}
    result = bosing.generate_waveforms(channels, shapes, schedule)
    w1 = result["xy"]
    w1 = w1[0] + 1j * w1[1]

    channels = {"xy": bosing.Channel(0, 2e9, 1000)}
    result = bosing.generate_waveforms(channels, shapes, schedule)
    w2 = result["xy"]
    w2 = w2[0] + 1j * w2[1]
    w2 = w2 * np.exp(1j * (2 * np.pi * freq * np.arange(1000) / 2e9))

    assert np.allclose(w1, w2)


def test_states():
    channels = {
        "xy0": bosing.Channel(100e6, 2e9, 1000),
        "xy1": bosing.Channel(50e6, 2e9, 1000),
    }
    schedule = bosing.Stack(duration=500e-9).with_children(
        bosing.Play("xy0", "hann", 0.3, 100e-9),
        bosing.Play("xy1", "hann", 0.5, 200e-9),
        bosing.ShiftPhase("xy0", 0.1),
        bosing.ShiftFreq("xy1", 10e6),
        bosing.Barrier(duration=10e-9),
    )
    shapes = {"hann": bosing.Hann()}
    _, states = bosing.generate_waveforms_with_states(channels, shapes, schedule, states=None)
    assert states["xy0"].base_freq == 100e6
    assert states["xy0"].delta_freq == 0
    assert states["xy0"].phase == 0.1
    assert states["xy1"].base_freq == 50e6
    assert states["xy1"].delta_freq == 10e6
    assert states["xy1"].phase_at(490e-9) == 50e6 * 490e-9
    shifted_states = {n: s.with_time_shift(500e-9) for n, s in states.items()}
    _, states = bosing.generate_waveforms_with_states(channels, shapes, schedule, states=shifted_states)
    assert states["xy0"].base_freq == 100e6
    assert states["xy0"].delta_freq == 0
    assert states["xy0"].phase == 0.2 + 100e6 * 500e-9
    assert states["xy1"].base_freq == 50e6
    assert states["xy1"].delta_freq == 20e6
    assert states["xy1"].phase_at(490e-9) == 50e6 * 490e-9 + 60e6 * 500e-9
