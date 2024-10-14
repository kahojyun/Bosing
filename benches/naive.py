# pyright: reportMissingTypeStubs=false
# pyright: reportUnknownVariableType=false
# pyright: reportUnknownMemberType=false
# pyright: reportUnknownArgumentType=false
import time
from collections.abc import Sequence
from math import ceil, floor

import numpy as np
import numpy.typing as npt
from scipy import signal


def get_biquad(
    amp: Sequence[float],
    tau: Sequence[float],
    fs: float,
) -> npt.NDArray[np.float64]:
    z = [-1 / (t * (1 + a)) for (a, t) in zip(amp, tau)]
    p = [-1 / t for t in tau]
    k = np.prod([1 + a for a in amp])
    z, p, k = signal.bilinear_zpk(z, p, k, fs)
    return signal.zpk2sos(p, z, 1 / k)


def get_slice(t0: float, width: float, sr: float) -> slice:
    i0 = floor(t0 * sr)
    i1 = ceil((t0 + width) * sr)
    return slice(i0, i1)


def x_mask(x: npt.NDArray[np.float64]) -> npt.NDArray[np.bool]:
    return (x > -0.5) & (x < 0.5)  # noqa: PLR2004


def hann_shape(x: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
    return 0.5 * (1 + np.cos(2 * np.pi * x))


def halfcos_shape(x: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
    return np.cos(np.pi * x)


def add_xy_pulse(
    x: npt.NDArray[np.float64],
    w: npt.NDArray[np.complex128],
    p: npt.NDArray[np.float64],
) -> None:
    w += np.piecewise(x, [x_mask(x)], [hann_shape]) * np.exp(1j * p)


def add_z_pulse(
    x: npt.NDArray[np.float64],
    w: npt.NDArray[np.float64],
) -> None:
    w += np.piecewise(x, [x_mask(x)], [halfcos_shape])


def gen_n(n: int) -> None:
    t0 = time.perf_counter()
    sample_rate = 2e9
    nxy = 64
    nu = 2 * nxy
    iir = get_biquad([0.1, -0.1], [100e-9, 1e-6], sample_rate)
    fir = [1, 0.1, 0.01, 0.001]
    length = 100000

    waveforms = {}
    t_axis = np.arange(length) / sample_rate
    width = 50e-9
    gap = 10e-9
    for i in range(nxy):
        w = np.zeros(length, dtype=np.complex128)
        phases = t_axis * (1e6 * i)
        for j in range(n):
            pulse_start = gap + (width + gap) * j
            pulse_center = pulse_start + width / 2
            s = get_slice(pulse_start, width, sample_rate)
            w_slice = w[s]
            t_slice = t_axis[s]
            x_slice = (t_slice - pulse_center) / width
            p_slice = phases[s]
            add_xy_pulse(x_slice, w_slice, p_slice)
        waveforms[f"xy{i}"] = w

    for i in range(nu):
        w = np.zeros(length, dtype=np.float64)
        for j in range(n):
            pulse_start = gap + (width + gap) * j
            pulse_center = pulse_start + width / 2
            s = get_slice(pulse_start, width, sample_rate)
            w_slice = w[s]
            t_slice = t_axis[s]
            x_slice = (t_slice - pulse_center) / width
            add_z_pulse(x_slice, w_slice)
        w = signal.sosfilt(iir, w)
        w = signal.convolve(w, fir)[: len(w)]
        waveforms[f"u{i}"] = w

    ct_matrix = np.eye(nu, dtype=np.float64)
    ct_matrix += 0.1
    ct_input = [waveforms[f"u{i}"] for i in range(nu)]
    ct_output = ct_matrix @ ct_input
    for i in range(nu):
        waveforms[f"u{i}"] = ct_output[i]

    t1 = time.perf_counter()
    print(f"Time: {t1-t0:.3f}s")


def main() -> None:
    for _ in range(10):
        gen_n(349)


if __name__ == "__main__":
    main()
