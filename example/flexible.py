import matplotlib.pyplot as plt

from bosing import Barrier, Channel, Grid, Hann, Play, Repeat, Stack, generate_waveforms

channels = [Channel("xy", 30e6, 2e9, 1000), Channel("u", 0, 2e9, 1000)]
shapes = [Hann()]
grid = Grid(columns=[40e-9, "auto", 40e-9]).with_children(
    # flexible u pulse spanning 3 columns
    (
        Play(
            channel_id=1,
            shape_id=0,
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
                channel_id=0,
                shape_id=0,
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
plt.plot(w.real, label="xy I")
plt.plot(w.imag, label="xy Q")
w = result["u"]
plt.plot(w.real, label="u I")
plt.plot(w.imag, label="u Q")
plt.legend()
