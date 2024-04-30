import matplotlib.pyplot as plt

from bosing import Barrier, Channel, Hann, Play, Stack, generate_waveforms

channels = {"xy": Channel(30e6, 2e9, 1000)}
shapes = {"hann": Hann()}
schedule = Stack(duration=500e-9).with_children(
    Play(
        channel_id="xy",
        shape_id="hann",
        amplitude=0.3,
        width=100e-9,
        plateau=200e-9,
    ),
    Barrier(duration=10e-9),
)
result = generate_waveforms(channels, shapes, schedule)
w = result["xy"]
plt.plot(w[0], label="I")
plt.plot(w[1], label="Q")
plt.legend()
