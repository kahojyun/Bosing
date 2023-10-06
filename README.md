# Qynit.PulseGen

Waveform generator for superconducting circuits.

> **Note:** This package is still under development and the API may change.

> **Related projects:** [pulsegen-client](https://github.com/kahojyun/pulsegen-client)

## Usage

> **Note:** This package requires .NET 7 or higher. Install the latest .NET SDK from [here](https://dotnet.microsoft.com/download/dotnet/7.0).

First, clone the repository:

```bash
git clone https://github.com/kahojyun/Qynit.PulseGen.git
```

Then, build the project:

```bash
cd Qynit.PulseGen
dotnet build
```

Finally, run the server:

```bash
dotnet run --project src/Qynit.PulseGen.Server
```

You can specify the url and port number to listen to:

```bash
dotnet run --project src/Qynit.PulseGen.Server --urls http://localhost:5200
```

## Waveform Viewer

The server provides a simple waveform viewer. To use the viewer, first install Node.js and npm from [here](https://nodejs.org/en/download/).

Then, install the dependencies:

```bash
cd src/Qynit.PulseGen.Server
npm install
npm build
``` 

When the server is running, open the viewer in your browser with the url `http://localhost:5200`.

The viewer uses [SciChart.js](https://www.scichart.com/) for plotting. The community edition of SciChart.js is free for **non-commercial** use.

## Development

Use Visual Studio or Visual Studio Code with the [C# extension](https://marketplace.visualstudio.com/items?itemName=ms-dotnettools.csharp).