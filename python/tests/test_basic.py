from typing import cast

import numpy as np
import numpy.typing as npt
import pytest
from rich.pretty import pretty_repr

import bosing


def _waveforms_from_instructions(
    channels: dict[str, bosing.Channel],
    envelopes: list[npt.NDArray[np.float64]],
    instructions: dict[str, list[bosing.Instruction]],
) -> dict[str, npt.NDArray[np.float64]]:
    result: dict[str, npt.NDArray[np.float64]] = {}
    for name, channel in channels.items():
        n_rows = 1 if channel.is_real else 2
        length = channel.length
        assert length is not None
        waveform = np.zeros((n_rows, length), dtype=np.float64)
        dt = 1.0 / channel.sample_rate
        for inst in instructions[name]:
            env = envelopes[inst.env_id]
            env_len = len(env)
            end = inst.i_start + env_len
            indices = np.arange(env_len, dtype=np.float64)
            phase = inst.phase + inst.freq * (indices * dt)
            carrier = np.exp(1j * (2 * np.pi * phase))
            samples = inst.amplitude * env * carrier
            waveform[0, inst.i_start : end] += samples.real
            if not channel.is_real:
                waveform[1, inst.i_start : end] += samples.imag
        result[name] = waveform
    return result


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
    wc = np.asarray(cast("npt.ArrayLike", w[0] + 1j * w[1]), dtype=np.complex128)
    assert wc[0] == 0
    assert wc[-1] == 0
    assert np.count_nonzero(wc) > 0


def test_auto_length() -> None:
    sample_rate = 2e9
    channels = {"xy": bosing.Channel(100e6, sample_rate, None)}
    shapes = {"hann": bosing.Hann()}
    schedule = bosing.Stack(duration=500e-9).with_children(
        bosing.Play("xy", "hann", 0.1, 100e-9),
    )

    result = bosing.generate_waveforms(channels, shapes, schedule)
    waveform = np.asarray(result["xy"], dtype=np.float64)

    assert channels["xy"].length is None
    assert waveform.shape == (2, 1000)
    assert np.count_nonzero(waveform) > 0
    assert np.count_nonzero(waveform[:, :800]) == 0
    assert np.count_nonzero(waveform[:, 800:]) > 0


def test_auto_length_uses_each_channel_sample_rate() -> None:
    schedule = bosing.Barrier(duration=100e-9)
    channels = {
        "slow": bosing.Channel(0, 1e9),
        "fast": bosing.Channel(0, 2e9),
    }

    result = bosing.generate_waveforms(channels, {}, schedule)

    assert result["slow"].shape == (2, 100)
    assert result["fast"].shape == (2, 200)


def test_auto_length_does_not_expand_for_delay_and_crosstalk() -> None:
    sample_rate = 1e9
    channels = {
        "source": bosing.Channel(0, sample_rate),
        "target": bosing.Channel(0, sample_rate, delay=10e-9),
    }
    shapes = {"hann": bosing.Hann()}
    schedule = bosing.Stack(
        bosing.Play("source", "hann", 0.1, 100e-9),
    )
    crosstalk = (
        np.array([[1.0, 0.0], [1.0, 0.0]]),
        ["source", "target"],
    )

    with pytest.raises(
        RuntimeError,
        match=r"Pulse sample range 10\.\.110 exceeds waveform length 100",
    ):
        _ = bosing.generate_waveforms(
            channels,
            shapes,
            schedule,
            crosstalk=crosstalk,
        )


def test_sampler_bounds_errors_include_required_and_available_lengths() -> None:
    shapes = {"hann": bosing.Hann()}

    with pytest.raises(
        RuntimeError,
        match=r"Pulse sample range 0\.\.100 exceeds waveform length 50",
    ):
        _ = bosing.generate_waveforms(
            {"xy": bosing.Channel(0, 1e9, 50)},
            shapes,
            bosing.Play("xy", "hann", 0.1, 100e-9),
        )

    with pytest.raises(
        RuntimeError,
        match=r"Pulse start index 100 is outside waveform length 50",
    ):
        _ = bosing.generate_waveforms(
            {"xy": bosing.Channel(0, 1e9, 50)},
            shapes,
            bosing.Absolute().with_children(
                (100e-9, bosing.Play("xy", "hann", 0.1, 10e-9))
            ),
        )

    with pytest.raises(RuntimeError, match="before the waveform start"):
        _ = bosing.generate_waveforms(
            {"xy": bosing.Channel(0, 1e9, 100, delay=-10e-9)},
            shapes,
            bosing.Play("xy", "hann", 0.1, 100e-9, alignment="start"),
        )


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
    wc1 = np.asarray(cast("npt.ArrayLike", w1[0] + 1j * w1[1]), dtype=np.complex128)

    channels = {"xy": bosing.Channel(0, sample_rate, length)}
    result = bosing.generate_waveforms(channels, shapes, schedule)
    w2 = result["xy"]
    wc2 = np.asarray(cast("npt.ArrayLike", w2[0] + 1j * w2[1]), dtype=np.complex128)
    wc2 = wc2 * np.exp(1j * (2 * np.pi * freq * np.arange(length) / sample_rate))

    assert np.allclose(wc1, wc2)


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


def test_measure() -> None:
    inner_duration = 10
    margin = 10
    schedule = bosing.Stack(bosing.Barrier(duration=inner_duration), margin=margin)

    measure_result = schedule.measure()

    assert measure_result == inner_duration + 2 * margin


def test_repr() -> None:
    c = bosing.Channel(2e9, 2e9, 1000, fir=[1, 2, 3])
    assert (
        pretty_repr(c)
        == "Channel(2000000000.0, 2000000000.0, 1000, fir=array([1., 2., 3.]))"
    )
    assert (
        repr(c) == "Channel(2000000000.0, 2000000000.0, 1000, fir=array([1., 2., 3.]))"
    )
    assert repr(bosing.Channel(2e9, 2e9)) == "Channel(2000000000.0, 2000000000.0, None)"


def test_generate_envelopes_and_instructions() -> None:
    length = 1000
    channels = {"xy": bosing.Channel(30e6, 2e9, length)}
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

    envelopes, instructions = bosing.generate_envelopes_and_instructions(
        channels,
        shapes,
        schedule,
    )
    assert len(envelopes) >= 1
    assert "xy" in instructions
    assert len(instructions["xy"]) >= 1

    for inst in instructions["xy"]:
        assert 0 <= inst.i_start < length
        assert 0 <= inst.env_id < len(envelopes)
        assert inst.amplitude >= 0
        assert np.isfinite(inst.freq)
        assert np.isfinite(inst.phase)

    waveforms_from_inst = _waveforms_from_instructions(
        channels,
        [np.asarray(env) for env in envelopes],
        instructions,
    )
    waveforms = bosing.generate_waveforms(channels, shapes, schedule)
    assert np.allclose(waveforms_from_inst["xy"], waveforms["xy"], atol=1e-12)

    states = {"xy": bosing.OscState(30e6, 5e6, 0.2)}
    envelopes, instructions = bosing.generate_envelopes_and_instructions(
        channels,
        shapes,
        schedule,
        states=states,
    )
    waveforms_from_inst = _waveforms_from_instructions(
        channels,
        [np.asarray(env) for env in envelopes],
        instructions,
    )
    waveforms, _ = bosing.generate_waveforms_with_states(
        channels,
        shapes,
        schedule,
        states=states,
    )
    assert np.allclose(waveforms_from_inst["xy"], waveforms["xy"], atol=1e-12)


def test_generate_envelopes_and_instructions_with_auto_length() -> None:
    channels = {"xy": bosing.Channel(30e6, 2e9)}
    shapes = {"hann": bosing.Hann()}
    schedule = bosing.Play("xy", "hann", 0.3, 100e-9)

    envelopes, instructions = bosing.generate_envelopes_and_instructions(
        channels,
        shapes,
        schedule,
    )

    assert len(envelopes) == 1
    assert len(instructions["xy"]) == 1
