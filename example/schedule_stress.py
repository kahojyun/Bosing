"""An example of using bosing to generate a pulse sequence."""

import time
from itertools import cycle

import numpy as np
from scipy.interpolate import make_interp_spline

from bosing import Absolute, Barrier, Channel, Hann, Interp, Play, Stack, generate_waveforms


def gen_n(n: int):
    t0 = time.perf_counter()
    nxy = 64
    nu = 2 * nxy
    nm = nxy // 8
    channels = (
        [Channel(f"xy{i}", 3e6 * i, 2e9, 100000) for i in range(nxy)]
        + [Channel(f"u{i}", 0, 2e9, 100000) for i in range(nu)]
        + [Channel(f"m{i}", 0, 2e9, 100000) for i in range(nm)]
    )
    c = {ch.name: i for i, ch in enumerate(channels)}
    halfcos = np.sin(np.linspace(0, np.pi, 10))
    spline = make_interp_spline(np.linspace(-0.5, 0.5, 10), halfcos)
    shapes = [
        Hann(),
        Interp(spline.t, spline.c, spline.k),
    ]
    s = {"hann": 0, "rect": None, "halfcos": 1}

    measure = Absolute().with_children(
        *(Play(c[f"m{i}"], s["hann"], 0.1, 30e-9, plateau=1e-6, frequency=20e6 * i) for i in range(nm))
    )
    c_group = Stack().with_children(*(Play(c[f"u{i}"], s["halfcos"], 0.01 * (i + 1), 50e-9) for i in range(nu)))
    x_group = Stack().with_children(
        *(Play(c[f"xy{i}"], s["hann"], 0.01 * (i + 1), 50e-9, drag_coef=5e-10) for i in range(nxy))
    )

    schedule = Stack(duration=50e-6).with_children(
        *(
            Stack().with_children(
                x_group,
                Barrier(duration=15e-9),
                c_group,
                Barrier(duration=15e-9),
            )
            for _ in range(n)
        ),
        measure,
        Barrier(duration=15e-9),
    )

    _ = generate_waveforms(channels, shapes, schedule)

    t1 = time.perf_counter()
    print(f"Time: {t1-t0:.3f}s")


def main():
    for i in cycle(range(1, 100)):
        print(i)
        gen_n(i)


if __name__ == "__main__":
    main()
