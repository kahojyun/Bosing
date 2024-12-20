# Bosing

[![Documentation Status](https://readthedocs.org/projects/bosing/badge/?version=latest)](https://bosing.readthedocs.io/zh-cn/latest/?badge=latest)
[![PyPI - Version](https://img.shields.io/pypi/v/bosing)](https://pypi.org/project/bosing/)

Waveform generator for superconducting circuits.

## Installation

```bash
pip install bosing
```

## Documentation

Docs are hosted on [Read the Docs](http://bosing.readthedocs.io/)

## Usage

Examples can be found in `python/examples`.

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

## Performance

`python/examples/schedule_stress.py` (0.15 s) vs `python/benches/naive.py` (1.4 s)

CPU: AMD Ryzen 5 5600

## Development

### Prerequisites

* Rustup for rust toolchain management.
* [maturin](https://github.com/PyO3/maturin) 1.7+.
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
