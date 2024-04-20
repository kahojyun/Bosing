import matplotlib.pyplot as plt

from bosing import Absolute, Barrier, Channel, Hann, Play, Stack, generate_waveforms

channels = [Channel("m", 0, 2e9, 1000)]
shapes = [Hann()]
measure = Absolute().with_children(
    *[
        Play(
            channel_id=0,
            shape_id=0,
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
plt.plot(w.real, label="I")
plt.plot(w.imag, label="Q")
plt.legend()
