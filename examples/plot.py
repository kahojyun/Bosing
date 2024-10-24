import matplotlib.pyplot as plt

from bosing import Barrier, Play, ShiftPhase, Stack

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

schedule.plot()
plt.show()
