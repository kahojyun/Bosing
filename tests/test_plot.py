import matplotlib as mpl

mpl.use("Agg")
from matplotlib.figure import Figure

from bosing import Barrier, Play, ShiftPhase, Stack


def test_plot() -> None:
    xy = [Play(f"xy{i}", "hann", 1.0, 100e-9) for i in range(2)]
    z = [
        Stack(Play(f"z{i}", "hann", 1.0, 100e-9), ShiftPhase(f"xy{i}", 1.0))
        for i in range(2)
    ]
    m = Stack(*(Play(f"m{i}", "hann", 1.0, 100e-9, plateau=200e-9) for i in range(2)))
    b = Barrier()

    schedule = Stack(
        xy[0],
        xy[1],
        b,
        z[1],
        b,
        xy[1],
        b,
        m,
    )

    fig = Figure()
    ax = fig.subplots()  # pyright: ignore[reportUnknownMemberType]
    _ = schedule.plot(ax)
