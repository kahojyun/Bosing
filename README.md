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

## Development

Use Visual Studio or Visual Studio Code with the [C# extension](https://marketplace.visualstudio.com/items?itemName=ms-dotnettools.csharp).