简单示例
========

简单的 Hann 脉冲
----------------

.. plot::
    :include-source:

    from bosing import *
    import matplotlib.pyplot as plt
    channels = [Channel("xy", 30e6, 2e9, 1000)]
    shapes = [Hann()]
    schedule = Stack(duration=500e-9).with_children(
        Play(
            channel_id=0,
            amplitude=0.3,
            shape_id=0,
            width=100e-9,
            plateau=200e-9,
        ),
        Barrier(duration=10e-9),
    )
    result = generate_waveforms(channels, shapes, schedule)
    i, q = result["xy"]
    plt.plot(i, label="I")
    plt.plot(q, label="Q")
    plt.legend()


插值包络
--------

.. plot::
    :include-source:

    from bosing import *
    import numpy as np
    import matplotlib.pyplot as plt
    channels = [Channel("xy", 0, 2e9, 1000)]
    # x should be in the range [-0.5, 0.5]
    x = np.linspace(-0.5, 0.5, 20)
    y = np.cos(np.pi * x)
    shapes = [Interp(x, y)]
    schedule = Stack(duration=500e-9).with_children(
        Play(
            channel_id=0,
            amplitude=0.3,
            shape_id=0,
            width=100e-9,
        ),
        Barrier(duration=10e-9),
    )
    result = generate_waveforms(channels, shapes, schedule)
    i, q = result["xy"]
    plt.plot(i, label="I")
    plt.plot(q, label="Q")
    plt.legend()


重叠脉冲
--------

.. plot::
    :include-source:

    from bosing import *
    import matplotlib.pyplot as plt
    channels = [Channel("m", 0, 2e9, 1000)]
    shapes = [Hann()]
    measure = Absolute().with_children(
        *[
            Play(
                channel_id=0,
                amplitude=0.3,
                shape_id=0,
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
    i, q = result["m"]
    plt.plot(i, label="I")
    plt.plot(q, label="Q")
    plt.legend()


变长脉冲
--------

.. plot::
    :include-source:

    from bosing import *
    import matplotlib.pyplot as plt
    channels = [Channel("xy", 30e6, 2e9, 1000), Channel("u", 0, 2e9, 1000)]
    shapes = [Hann()]
    grid = Grid(columns=[40e-9, "auto", 40e-9]).with_children(
        # flexible u pulse spanning 3 columns
        (0, 3, Play(
            channel_id=1,
            amplitude=0.5,
            shape_id=0,
            width=60e-9,
            alignment="stretch",
            flexible=True,
        )),
        # xy pulse in the middle column
        (1, Repeat(
            Play(
                channel_id=0,
                amplitude=0.3,
                shape_id=0,
                width=60e-9,
            ),
            count=3,
            spacing=30e-9,
        )),
    )
    schedule = Stack(duration=500e-9).with_children(
        grid,
        Barrier(duration=10e-9),
    )
    result = generate_waveforms(channels, shapes, schedule)
    i, q = result["xy"]
    plt.plot(i, label="xy I")
    plt.plot(q, label="xy Q")
    i, q = result["u"]
    plt.plot(i, label="u I")
    plt.plot(q, label="u Q")
    plt.legend()
