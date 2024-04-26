import matplotlib.pyplot as plt

from bosing import Barrier, Channel, Grid, Hann, Play, Repeat, Stack, generate_waveforms

channels = {
    "xy": Channel(30e6, 2e9, 1000),
    "u": Channel(0, 2e9, 1000),
}
shapes = {
    "hann": Hann(),
}
grid = Grid(columns=[40e-9, "auto", 40e-9]).with_children(
    # flexible u pulse spanning 3 columns
    (
        Play(
            channel_id="u",
            shape_id="hann",
            amplitude=0.5,
            width=60e-9,
            alignment="stretch",
            flexible=True,
        ),
        0,
        3,
    ),
    # xy pulse in the middle column
    (
        Repeat(
            Play(
                channel_id="xy",
                shape_id="hann",
                amplitude=0.3,
                width=60e-9,
            ),
            count=3,
            spacing=30e-9,
        ),
        1,
    ),
)
schedule = Stack(duration=500e-9).with_children(
    grid,
    Barrier(duration=10e-9),
)
result = generate_waveforms(channels, shapes, schedule)
w = result["xy"]
plt.plot(w[0], label="xy I")
plt.plot(w[1], label="xy Q")
w = result["u"]
plt.plot(w[0], label="u I")
plt.plot(w[1], label="u Q")
plt.legend()
