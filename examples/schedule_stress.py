"""An example of using bosing to generate a pulse sequence."""

import time
from collections.abc import Sequence
from itertools import cycle

import numpy as np
from scipy import signal
from scipy.interpolate import make_interp_spline

from bosing import (
    Absolute,
    Barrier,
    Channel,
    Hann,
    Interp,
    Play,
    Stack,
    generate_waveforms,
)


def get_biquad(
    amp: Sequence[float],
    tau: Sequence[float],
    fs: float,
) -> np.ndarray:
    z = [-1 / (t * (1 + a)) for (a, t) in zip(amp, tau)]
    p = [-1 / t for t in tau]
    k = np.prod([1 + a for a in amp])
    z, p, k = signal.bilinear_zpk(z, p, k, fs)
    return signal.zpk2sos(p, z, 1 / k)


def gen_n(n: int) -> None:
    t0 = time.perf_counter()
    nxy = 64
    nu = 2 * nxy
    nm = nxy // 8
    iir = get_biquad([0.1, -0.1], [100e-9, 1e-6], 2e9)
    fir = [1, 0.1, 0.01, 0.001]
    channels = (
        {
            f"xy{i}": Channel(
                3e6 * i,
                2e9,
                100000,
                iq_matrix=[[1.0, 0.1], [0.1, 1.0]],
                offset=[0.1, 0.2],
            )
            for i in range(nxy)
        }
        | {
            f"u{i}": Channel(0, 2e9, 100000, iir=iir, fir=fir, is_real=True)
            for i in range(nu)
        }
        | {f"m{i}": Channel(0, 2e9, 100000) for i in range(nm)}
    )
    halfcos = np.sin(np.linspace(0, np.pi, 10))
    spline = make_interp_spline(np.linspace(-0.5, 0.5, 10), halfcos)
    shapes = {
        "hann": Hann(),
        "halfcos": Interp(spline.t, spline.c, spline.k),
    }

    measure = Absolute().with_children(
        *(
            Play(f"m{i}", "hann", 0.1, 30e-9, plateau=1e-6, frequency=20e6 * i)
            for i in range(nm)
        ),
    )
    c_group = Stack().with_children(
        *(Play(f"u{i}", "halfcos", 0.01 * (i + 1), 50e-9) for i in range(nu)),
    )
    x_group = Stack().with_children(
        *(
            Play(f"xy{i}", "hann", 0.01 * (i + 1), 50e-9, drag_coef=5e-10)
            for i in range(nxy)
        ),
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

    ct_matrix = np.eye(nu, dtype=np.float64)
    ct_matrix += 0.1
    ct_names = [f"u{i}" for i in range(nu)]

    _ = generate_waveforms(channels, shapes, schedule, crosstalk=(ct_matrix, ct_names))

    t1 = time.perf_counter()
    print(f"Time: {t1-t0:.3f}s")


def main() -> None:
    for i in cycle(range(349, 350)):
        print(i)
        gen_n(i)


if __name__ == "__main__":
    main()
