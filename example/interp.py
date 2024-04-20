import matplotlib.pyplot as plt
import numpy as np
from scipy.interpolate import make_interp_spline

from bosing import Barrier, Channel, Interp, Play, Stack, generate_waveforms

channels = [Channel("xy", 0, 2e9, 1000)]
# x should be in the range [-0.5, 0.5]
x = np.linspace(-0.5, 0.5, 20)
y = np.cos(np.pi * x)
interp = make_interp_spline(x, y)
knots = interp.t
controls = interp.c
degree = interp.k
shapes = [Interp(knots, controls, degree)]
schedule = Stack(duration=500e-9).with_children(
    Play(
        channel_id=0,
        shape_id=0,
        amplitude=0.3,
        width=100e-9,
    ),
    Barrier(duration=10e-9),
)
result = generate_waveforms(channels, shapes, schedule)
w = result["xy"]
plt.plot(w.real, label="I")
plt.plot(w.imag, label="Q")
plt.legend()
