"""An example of using bosing to generate a pulse sequence."""

import numpy as np
from matplotlib import pyplot as plt
from scipy.interpolate import make_interp_spline

from bosing import Absolute, Barrier, Channel, Grid, Hann, Interp, Play, Repeat, ShiftPhase, Stack, generate_waveforms

if __name__ == "__main__":
    length = 100000
    channels = [
        Channel("xy0", 0, 2e9, length),
        Channel("xy1", 0, 2e9, length),
        Channel("u0", 0, 2e9, length),
        Channel("u1", 0, 2e9, length),
        Channel("m0", 0, 2e9, length),
    ]
    c = {ch.name: i for i, ch in enumerate(channels)}
    halfcos = np.sin(np.linspace(0, np.pi, 10))
    interp = make_interp_spline(np.linspace(-0.5, 0.5, 10), halfcos)
    shapes = [
        Hann(),
        Interp(interp.t, interp.c, interp.k),
    ]
    s = {"hann": 0, "rect": None, "halfcos": 1}

    measure = Absolute(
        Play(c["m0"], s["hann"], 0.1, 30e-9, plateau=1e-6, frequency=123e6),
        Play(c["m0"], s["hann"], 0.15, 30e-9, plateau=1e-6, frequency=-233e6),
    )
    c01 = Stack(
        Play(c["u0"], s["rect"], 0.5, 50e-9),
        Play(c["u1"], s["rect"], 0.5, 50e-9),
        ShiftPhase(c["xy0"], 0.1),
        ShiftPhase(c["xy1"], 0.2),
    )
    x0 = Play(c["xy0"], s["hann"], 0.3, 50e-9, drag_coef=5e-10)
    x1 = Play(c["xy1"], s["hann"], 0.4, 100e-9, drag_coef=3e-10)
    x_group = Grid(
        Stack(x0, alignment="center"),
        Stack(x1, alignment="center"),
    )

    schedule = Stack(duration=50e-6).with_children(
        Repeat(
            Stack(
                x_group,
                Barrier(duration=15e-9),
                c01,
            ),
            count=200,
            spacing=15e-9,
        ),
        Barrier(duration=15e-9),
        measure,
        Barrier(duration=15e-9),
    )

    result = generate_waveforms(channels, shapes, schedule, time_tolerance=1e-13)

    t = np.arange(length) / 2e9
    plt.plot(t, result["xy0"].real)
    plt.plot(t, result["xy0"].imag)
    plt.plot(t, result["xy1"].real)
    plt.plot(t, result["xy1"].imag)
    plt.plot(t, result["u1"].real)
    plt.plot(t, result["u1"].imag)
    plt.plot(t, result["m0"].real)
    plt.plot(t, result["m0"].imag)
    plt.show()
