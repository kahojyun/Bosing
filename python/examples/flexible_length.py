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
channels = {"xy": Channel(30e6, sample_rate, length=None)}
shapes = {"hann": Hann()}
result = generate_waveforms(channels, shapes, schedule)

# Automatic length covers the schedule's logical time window. These configurations
# exceed that window, so the sampler reports how many samples are required.
invalid_channels = {
    "explicit length too short": {"xy": Channel(30e6, sample_rate, length=100)},
    "delay moves pulse past the window": {
        "xy": Channel(30e6, sample_rate, delay=20e-9)
    },
}


def show_expected_error(label: str, invalid: dict[str, Channel]) -> None:
    try:
        _ = generate_waveforms(invalid, shapes, schedule)
    except RuntimeError as error:
        print(f"{label}:\n{error}")


for label, invalid in invalid_channels.items():
    show_expected_error(label, invalid)

w = result["xy"]
plt.plot(w[0], label="I")
plt.plot(w[1], label="Q")
plt.legend()
