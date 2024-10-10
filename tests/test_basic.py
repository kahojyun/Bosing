from typing import TYPE_CHECKING

import numpy as np

import bosing

if TYPE_CHECKING:
    from numpy.typing import NDArray


def test_basic() -> None:
    length = 100000
    channels = {"xy0": bosing.Channel(100e6, 2e9, length)}
    shapes = {"hann": bosing.Hann()}
    schedule = bosing.Stack(duration=49.9e-6).with_children(
        bosing.Play("xy0", "hann", 0.1, 100e-9),
    )
    result = bosing.generate_waveforms(channels, shapes, schedule)
    assert "xy0" in result
    w = result["xy0"]
    assert w.shape == (2, length)
    w: NDArray[np.float64] = w[0] + 1j * w[1]
    assert w[0] == 0
    assert w[-1] == 0
    assert np.any(w != 0)  # pyright: ignore[reportAny]


def test_mixing() -> None:
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
    length = 1000
    sample_rate = 2e9

    channels = {"xy": bosing.Channel(freq, sample_rate, length)}
    result = bosing.generate_waveforms(channels, shapes, schedule)
    w1 = result["xy"]
    w1: NDArray[np.float64] = w1[0] + 1j * w1[1]

    channels = {"xy": bosing.Channel(0, sample_rate, length)}
    result = bosing.generate_waveforms(channels, shapes, schedule)
    w2 = result["xy"]
    w2: NDArray[np.float64] = w2[0] + 1j * w2[1]
    w2 = w2 * np.exp(1j * (2 * np.pi * freq * np.arange(length) / sample_rate))

    assert np.allclose(w1, w2)


def test_states() -> None:
    length = 1000
    base_freq0 = 100e6
    base_freq1 = 50e6
    phase_shift = 0.1
    freq_shift = 10e6
    duration = 500e-9
    gap = 10e-9
    shift_instant = duration - gap
    channels = {
        "xy0": bosing.Channel(base_freq0, 2e9, length),
        "xy1": bosing.Channel(base_freq1, 2e9, length),
    }
    schedule = bosing.Stack(duration=duration).with_children(
        bosing.Play("xy0", "hann", 0.3, 100e-9),
        bosing.Play("xy1", "hann", 0.5, 200e-9),
        bosing.ShiftPhase("xy0", phase_shift),
        bosing.ShiftFreq("xy1", freq_shift),
        bosing.Barrier(duration=gap),
    )
    shapes = {"hann": bosing.Hann()}
    _, states = bosing.generate_waveforms_with_states(
        channels,
        shapes,
        schedule,
        states=None,
    )
    assert states["xy0"].base_freq == base_freq0
    assert states["xy0"].delta_freq == 0
    assert states["xy0"].phase == phase_shift
    assert states["xy1"].base_freq == base_freq1
    assert states["xy1"].delta_freq == freq_shift
    assert states["xy1"].phase_at(shift_instant) == base_freq1 * shift_instant
    shifted_states = {n: s.with_time_shift(duration) for n, s in states.items()}
    _, states = bosing.generate_waveforms_with_states(
        channels,
        shapes,
        schedule,
        states=shifted_states,
    )
    assert states["xy0"].base_freq == base_freq0
    assert states["xy0"].delta_freq == 0
    assert states["xy0"].phase == phase_shift * 2 + base_freq0 * duration
    assert states["xy1"].base_freq == base_freq1
    assert states["xy1"].delta_freq == freq_shift * 2
    assert (
        states["xy1"].phase_at(shift_instant)
        == base_freq1 * shift_instant + (base_freq1 + freq_shift) * duration
    )
