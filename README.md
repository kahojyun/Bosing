# Bosing

[![Documentation Status](https://readthedocs.org/projects/bosing/badge/?version=latest)](https://bosing.readthedocs.io/zh-cn/latest/?badge=latest)

Waveform generator for superconducting circuits.

## Installation

```bash
pip install bosing
```

## Documentation

Docs are hosted on [Read the Docs](http://bosing.readthedocs.io/)

## Usage

Examples can be found in `examples`.

```python
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
plt.show()
```

## Development

### Prerequisites

* Rustup for rust toolchain management.
* [maturin](https://github.com/PyO3/maturin) 1.5+.
* [uv](https://github.com/astral-sh/uv) for python project management.

```bash
git clone https://github.com/kahojyun/Bosing.git
cd Bosing
uv sync
uv run task makedocs # build docs
uv run task format # format rust and python code
uv run task lint # lint rust and python code
uv run task test # run cargo test and pytest
```
