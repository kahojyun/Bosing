import ctypes
import math
import threading
from contextlib import contextmanager
from ctypes import cdll
from itertools import cycle
from time import perf_counter, sleep
from typing import Dict, Tuple

import matplotlib.pyplot as plt
import numpy as np
from models import *
from scipy import signal

lib = cdll.LoadLibrary(
    r"E:\Qynit.PulseGen\artifacts\publish\Qynit.PulseGen.Aot\release_win-x64\Qynit.PulseGen.Aot.dll"
)

# int Qynit_PulseGen_Run(char* request, int length, void** out_handle)
Qynit_PulseGen_Run = lib.Qynit_PulseGen_Run
Qynit_PulseGen_Run.argtypes = [
    ctypes.c_char_p,
    ctypes.c_int,
    ctypes.POINTER(ctypes.c_void_p),
]
Qynit_PulseGen_Run.restype = ctypes.c_int

# int Qynit_PulseGen_CopyWaveform(void* handle, char* name, float* i, float* q, int length)
Qynit_PulseGen_CopyWaveform = lib.Qynit_PulseGen_CopyWaveform
Qynit_PulseGen_CopyWaveform.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.c_int,
]
Qynit_PulseGen_CopyWaveform.restype = ctypes.c_int

# int Qynit_PulseGen_FreeWaveform(void* handle)
Qynit_PulseGen_FreeWaveform = lib.Qynit_PulseGen_FreeWaveform
Qynit_PulseGen_FreeWaveform.argtypes = [ctypes.c_void_p]
Qynit_PulseGen_FreeWaveform.restype = ctypes.c_int


@contextmanager
def pulsegen(msg: bytes):
    handle = ctypes.c_void_p()
    ret = Qynit_PulseGen_Run(msg, len(msg), ctypes.byref(handle))
    if ret != 0:
        raise Exception(f"Failed to run PulseGen, error code: {ret}")
    try:
        yield handle
    finally:
        ret = Qynit_PulseGen_FreeWaveform(handle)
        if ret != 0:
            raise Exception(f"Failed to free waveform, error code: {ret}")


def run(request: Request) -> Dict[str, Tuple[np.ndarray, np.ndarray]]:
    msg = request.packb()
    with pulsegen(msg) as handle:
        waveforms = {}
        for ch in request.channels:
            wave_i = np.empty(ch.length, dtype=np.float32)
            wave_q = np.empty(ch.length, dtype=np.float32)
            pstr = ch.name.encode("utf-8")
            ptr_i_float = wave_i.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
            ptr_q_float = wave_q.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
            length = ch.length
            ret = Qynit_PulseGen_CopyWaveform(
                handle, pstr, ptr_i_float, ptr_q_float, length
            )
            if ret != 0:
                raise Exception(f"Failed to copy waveform, error code: {ret}")
            waveforms[ch.name] = (wave_i, wave_q)
        return waveforms


def get_biquad(amp, tau, fs):
    z = -1 / (tau * (1 + amp))
    p = -1 / tau
    k = 1 + amp
    z, p, k = signal.bilinear_zpk([z], [p], k, fs)
    sos = signal.zpk2sos(p, z, 1 / k)
    return Biquad(sos[0][0], sos[0][1], sos[0][2], sos[0][4], sos[0][5])


def get_iq_calibration(ratio, phase, offset_i, offset_q):
    return IqCalibration(
        1, -math.tan(phase), 0, ratio / math.cos(phase), offset_i, offset_q
    )


def gen_n(n: int):

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

    _ = run(job)


EXITING = False


def stress_main():
    for i in cycle(range(1, 100)):
        print(i)
        t0 = perf_counter()
        gen_n(i)
        t1 = perf_counter()
        print(f"Time: {t1-t0:.3f}s")
        if EXITING:
            break


def demo_main():
    bq = get_biquad(-0.1, 20e-9, 2e9)
    fir = signal.firwin(5, 100e6, fs=2e9)
    channels = [
        ChannelInfo(
            "xy0",
            0,
            2e9,
            0,
            100000,
            -10,
            iq_calibration=get_iq_calibration(1.1, math.pi / 3, 0, 0),
        ),
        ChannelInfo("xy1", 0, 2e9, 0, 100000, -10),
        ChannelInfo("u0", 0, 2e9, 0, 100000, -10, iir=[bq]),
        ChannelInfo("u1", 0, 2e9, 0, 100000, -10, iir=[bq], fir=fir),
        ChannelInfo("m0", 0, 2e9, 0, 100000, 0),
    ]
    c = {ch.name: i for i, ch in enumerate(channels)}
    halfcos = np.sin(np.linspace(0, np.pi, 10))
    shapes = [
        HannShape(),
        InterpolatedShape(np.linspace(-0.5, 0.5, 10), halfcos),
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

    job = Request(channels, shapes, schedule, options=options)

    t0 = perf_counter()
    result = run(job)
    t1 = perf_counter()
    print(t1 - t0)

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


if __name__ == "__main__":
    threads = [threading.Thread(target=stress_main) for _ in range(1)]
    for t in threads:
        t.start()
    try:
        while True:
            sleep(1)
    except KeyboardInterrupt:
        EXITING = True
        for t in threads:
            t.join()
    # demo_main()
