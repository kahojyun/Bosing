from bosing import (
    Barrier,
    Channel,
    Hann,
    Play,
    Stack,
    generate_envelopes_and_instructions,
)


def main() -> None:
    length = 1000
    channels = {"xy": Channel(30e6, 2e9, length)}
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

    envelopes, instructions = generate_envelopes_and_instructions(
        channels, shapes, schedule
    )

    inst0 = instructions["xy"][0]
    env0 = envelopes[inst0.env_id]

    print("env dtype:", env0.dtype)
    print("env length:", env0.shape[0])
    print(
        "inst:",
        {
            "i_start": inst0.i_start,
            "env_id": inst0.env_id,
            "amplitude": inst0.amplitude,
            "freq": inst0.freq,
            "phase": inst0.phase,
        },
    )


if __name__ == "__main__":
    main()
