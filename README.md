# Bosing

[![Documentation Status](https://readthedocs.org/projects/bosing/badge/?version=latest)](https://bosing.readthedocs.io/zh-cn/latest/?badge=latest)

Waveform generator for superconducting circuits.

## Installation

```bash
pip install bosing
```

## Documentation

Docs are hosted on [Read the Docs](http://bosing.readthedocs.io/)

## Development

### Prerequisites

* .NET 8 SDK. Install the latest .NET SDK from [here](https://dotnet.microsoft.com/download/dotnet) or install with Visual Studio.
* [hatch](https://github.com/pypa/hatch) for python project management.

### Development install

Ensure `dotnet` cli is in `PATH`.

```bash
git clone https://github.com/kahojyun/Bosing.git
cd Bosing
pip install -e .
```

### Build docs

```bash
hatch run docs:build
```

### Run tests

```bash
dotnet test
hatch run test:run
```

### Usage (TODO)

Examples can be found in `python/examples`.

```python
from bosing import Play, Hann, Channel, Stack, generate_waveforms
import matplotlib.pyplot as plt

channels = [Channel("xy", 200e6, 2e9, 100000)]
shapes = [Hann()]
schedule = Stack(duration=49.9e-6).with_children(
    Play(
        channel_id = 0,
        amplitude = 0.3,
        shape_id = 0,
        width = 100e-9,
    )
)
result = generate_waveforms(channels, shapes, schedule)
i, q = result["xy"]
plt.plot(i)
plt.plot(q)
plt.show()
```

### Tooling

Use Visual Studio or Visual Studio Code with the [C# extension](https://marketplace.visualstudio.com/items?itemName=ms-dotnettools.csharp).

Manage python project with [hatch](https://github.com/pypa/hatch).