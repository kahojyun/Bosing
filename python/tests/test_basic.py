import numpy as np

import bosing


def test_basic():
    channels = [bosing.ChannelInfo("xy0", 100e6, 2e9, 0, 100000, -10)]
    shapes = [bosing.HannShape()]
    schedule = bosing.Stack(duration=49.9e-6).with_children(
        bosing.Play(0, 0.1, 0, 100e-9)
    )
    req = bosing.Request(channels, shapes, schedule)
    result = bosing.generate_waveforms(req)
    assert "xy0" in result
    i, q = result["xy0"]
    assert len(i) == len(q)
    assert len(i) == 100000
    assert i[0] == 0
    assert i[-1] == 0
    assert np.any(i != 0)
    assert q[0] == 0
    assert q[-1] == 0
    assert np.any(q != 0)
