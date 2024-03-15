# Qynit.PulseGen

Waveform generator for superconducting circuits.

> **Note:** This package is still under development and the API may change.

## Prerequisites

* .NET 8 or higher. Install the latest .NET SDK from [here](https://dotnet.microsoft.com/download/dotnet) or install with Visual Studio.
* NodeJS

## Python API

### Build from source

.NET SDK and NodeJS is required.

```bash
git clone https://github.com/kahojyun/Qynit.PulseGen.git
cd Qynit.PulseGen
pip install .
```

### Usage (TODO)

Examples can be found in `python/examples`.

## Waveform Viewer

The server provides a simple waveform viewer. When the server is running, open the viewer in your browser with the url `http://localhost:{port}`.

The viewer uses [SciChart.js](https://www.scichart.com/) for plotting. The community edition of SciChart.js is free for **non-commercial** use.

## Development

Use Visual Studio or Visual Studio Code with the [C# extension](https://marketplace.visualstudio.com/items?itemName=ms-dotnettools.csharp).