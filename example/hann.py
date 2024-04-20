import matplotlib.pyplot as plt

from bosing import Barrier, Channel, Hann, Play, Stack, generate_waveforms

channels = [Channel("xy", 30e6, 2e9, 1000)]
shapes = [Hann()]
schedule = Stack(duration=500e-9).with_children(
    Play(
        channel_id=0,
        shape_id=0,
        amplitude=0.3,
        width=100e-9,
        plateau=200e-9,
    ),
    Barrier(duration=10e-9),
)
result = generate_waveforms(channels, shapes, schedule)
w = result["xy"]
plt.plot(w.real, label="I")
plt.plot(w.imag, label="Q")
plt.legend()
