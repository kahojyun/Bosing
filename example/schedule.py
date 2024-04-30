"""An example of using bosing to generate a pulse sequence."""

import numpy as np
from matplotlib import pyplot as plt
from scipy.interpolate import make_interp_spline

from bosing import Absolute, Barrier, Channel, Grid, Hann, Interp, Play, Repeat, ShiftPhase, Stack, generate_waveforms

if __name__ == "__main__":
    length = 100000
    channels = {
        "xy0": Channel(0, 2e9, length),
        "xy1": Channel(0, 2e9, length),
        "u0": Channel(0, 2e9, length),
        "u1": Channel(0, 2e9, length),
        "m0": Channel(0, 2e9, length),
    }
    halfcos = np.sin(np.linspace(0, np.pi, 10))
    interp = make_interp_spline(np.linspace(-0.5, 0.5, 10), halfcos)
    shapes = {
        "hann": Hann(),
        "halfcos": Interp(interp.t, interp.c, interp.k),
    }

    measure = Absolute(
        Play("m0", "hann", 0.1, 30e-9, plateau=1e-6, frequency=123e6),
        Play("m0", "hann", 0.15, 30e-9, plateau=1e-6, frequency=-233e6),
    )
    c01 = Stack(
        Play("u0", None, 0.5, 50e-9),
        Play("u1", None, 0.5, 50e-9),
        ShiftPhase("xy0", 0.1),
        ShiftPhase("xy1", 0.2),
    )
    x0 = Play("xy0", "hann", 0.3, 50e-9, drag_coef=5e-10)
    x1 = Play("xy1", "hann", 0.4, 100e-9, drag_coef=3e-10)
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
    plt.plot(t, result["xy0"][0])
    plt.plot(t, result["xy0"][1])
    plt.plot(t, result["xy1"][0])
    plt.plot(t, result["xy1"][1])
    plt.plot(t, result["u1"][0])
    plt.plot(t, result["u1"][1])
    plt.plot(t, result["m0"][0])
    plt.plot(t, result["m0"][1])
    plt.show()
