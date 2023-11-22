"""An example of using the client to generate a pulse sequence.

.. note::

    The server must be running for this example to work.
"""

from itertools import cycle
import time

import numpy as np

from pulsegen_client import *


def gen_n(n: int):
    t0 = time.perf_counter()
    nxy = 64
    nu = 2 * nxy
    nm = nxy // 8
    channels = (
        [ChannelInfo(f"xy{i}", 3e6 * i, 2e9, 0, 100000, -10) for i in range(nxy)]
        + [ChannelInfo(f"u{i}", 0, 2e9, 0, 100000, -10) for i in range(nu)]
        + [ChannelInfo(f"m{i}", 0, 2e9, 0, 100000, 0) for i in range(nm)]
    )
    c = {ch.name: i for i, ch in enumerate(channels)}
    halfcos = np.sin(np.linspace(0, np.pi, 10))
    shapes = [
        HannShape(),
        InterpolatedShape(np.linspace(-0.5, 0.5, 10), halfcos),
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

    job = Request(channels, shapes, schedule)

    _ = generate_waveforms(job)

    t1 = time.perf_counter()
    print(f"Time: {t1-t0:.3f}s")


def main():
    for i in cycle(range(1, 100)):
        print(i)
        gen_n(i)


if __name__ == "__main__":
    start_server()
    main()
