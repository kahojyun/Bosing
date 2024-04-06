"""An example of using bosing to generate a pulse sequence.
"""

import time
from itertools import cycle

import numpy as np

from bosing import *


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
    shapes = [
        Hann(),
        Interp(np.linspace(-0.5, 0.5, 10), halfcos),
    ]
    s = {"hann": 0, "rect": -1, "halfcos": 1}

    measure = Absolute().with_children(
        *(
            Play(c[f"m{i}"], 0.1, s["hann"], 30e-9, plateau=1e-6, frequency=20e6 * i)
            for i in range(nm)
        )
    )
    c_group = Stack().with_children(
        *(Play(c[f"u{i}"], 0.01 * (i + 1), s["halfcos"], 50e-9) for i in range(nu))
    )
    x_group = Stack().with_children(
        *(
            Play(c[f"xy{i}"], 0.01 * (i + 1), s["hann"], 50e-9, drag_coef=5e-10)
            for i in range(nxy)
        )
    )

    schedule = Stack(duration=49.9e-6).with_children(
        Repeat(
            Stack().with_children(
                x_group,
                Barrier(duration=15e-9),
                c_group,
            ),
            count=n,
            spacing=15e-9,
        ),
        Barrier(duration=15e-9),
        measure,
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
