import matplotlib.pyplot as plt

from bosing import Channel, Hann, Play, Stack, generate_waveforms

sample_rate = 2e9
schedule = Stack(
    Play(
        channel_id="xy",
        shape_id="hann",
        amplitude=0.3,
        width=100e-9,
        plateau=200e-9,
    ),
    margin=10e-9,
)
channels = {"xy": Channel(30e6, sample_rate, int(sample_rate * schedule.measure()))}
shapes = {"hann": Hann()}
result = generate_waveforms(channels, shapes, schedule)
w = result["xy"]
plt.plot(w[0], label="I")
plt.plot(w[1], label="Q")
plt.legend()
