"""An example of using bosing to generate a pulse sequence.
"""

import math
from time import perf_counter

import numpy as np
from matplotlib import pyplot as plt
from scipy import signal

from bosing import *


def get_biquad(amp, tau, fs):
    z = -1 / (tau * (1 + amp))
    p = -1 / tau
    k = 1 + amp
    z, p, k = signal.bilinear_zpk([z], [p], k, fs)
    sos = signal.zpk2sos(p, z, 1 / k)
    return Biquad(sos[0][0], sos[0][1], sos[0][2], sos[0][4], sos[0][5])


def get_iq_calibration(ratio, phase, offset_i, offset_q):
    return IqCali(1, -math.tan(phase), 0, ratio / math.cos(phase), offset_i, offset_q)


if __name__ == "__main__":
    t0 = perf_counter()

    bq = get_biquad(-0.1, 20e-9, 2e9)
    fir = signal.firwin(5, 100e6, fs=2e9)
    channels = [
        Channel(
            "xy0",
            0,
            2e9,
            100000,
            iq_calibration=get_iq_calibration(1.1, math.pi / 3, 0, 0),
        ),
        Channel("xy1", 0, 2e9, 100000),
        Channel("u0", 0, 2e9, 100000, iir=[bq]),
        Channel("u1", 0, 2e9, 100000, iir=[bq], fir=fir),
        Channel("m0", 0, 2e9, 100000),
    ]
    c = {ch.name: i for i, ch in enumerate(channels)}
    halfcos = np.sin(np.linspace(0, np.pi, 10))
    shapes = [
        Hann(),
        Interp(np.linspace(-0.5, 0.5, 10), halfcos),
    ]
    s = {"hann": 0, "rect": -1, "halfcos": 1}

    measure = Absolute().with_children(
        Play(c["m0"], 0.1, s["hann"], 30e-9, plateau=1e-6, frequency=123e6),
        Play(c["m0"], 0.15, s["hann"], 30e-9, plateau=1e-6, frequency=-233e6),
    )
    c01 = Stack().with_children(
        Play(c["u0"], 0.5, s["rect"], 50e-9),
        Play(c["u1"], 0.5, s["rect"], 50e-9),
        ShiftPhase(c["xy0"], 0.1),
        ShiftPhase(c["xy1"], 0.2),
    )
    x0 = Play(c["xy0"], 0.3, s["hann"], 50e-9, drag_coef=5e-10)
    x1 = Play(c["xy1"], 0.4, s["hann"], 100e-9, drag_coef=3e-10)
    x_group = Grid().with_children(
        Stack([x0], alignment="center"),
        Stack([x1], alignment="center"),
    )

    schedule = Stack(duration=49.9e-6).with_children(
        Repeat(
            Stack().with_children(
                x_group,
                Barrier(duration=15e-9),
                c01,
            ),
            count=200,
            spacing=15e-9,
        ),
        Barrier(duration=15e-9),
        measure,
    )

    options = Options(time_tolerance=1e-13)

    t1 = perf_counter()

    result = generate_waveforms(channels, shapes, schedule, options=options)

    t2 = perf_counter()

    print(f"Build time: {t1 - t0:.3f}s")
    print(f"Run time: {t2 - t1:.3f}s")

    t = np.arange(100000) / 2e9
    plt.plot(t, result["xy0"][0])
    plt.plot(t, result["xy0"][1])
    plt.plot(t, result["xy1"][0])
    plt.plot(t, result["xy1"][1])
    plt.plot(t, signal.lfilter(fir, [1], result["u0"][0]))
    plt.plot(t, signal.lfilter(fir, [1], result["u0"][1]))
    plt.plot(t, result["u1"][0])
    plt.plot(t, result["u1"][1])
    plt.plot(t, result["m0"][0])
    plt.plot(t, result["m0"][1])
    plt.show()
