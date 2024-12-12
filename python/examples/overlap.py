import matplotlib.pyplot as plt

from bosing import Absolute, Barrier, Channel, Hann, Play, Stack, generate_waveforms

channels = {"m": Channel(30e6, 2e9, 1000)}
shapes = {"hann": Hann()}
measure = Absolute().with_children(
    *[
        Play(
            channel_id="m",
            shape_id="hann",
            amplitude=0.3,
            width=100e-9,
            plateau=300e-9,
            frequency=40e6 * i + 60e6,
        )
        for i in range(2)
    ]
)
schedule = Stack(duration=500e-9).with_children(
    measure,
    Barrier(duration=10e-9),
)
result = generate_waveforms(channels, shapes, schedule)
w = result["m"]
plt.plot(w[0], label="I")
plt.plot(w[1], label="Q")
plt.legend()
