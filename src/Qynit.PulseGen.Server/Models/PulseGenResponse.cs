using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed record PulseGenResponse(
    [property: Key(0)] IList<PooledComplexArray<double>> Waveforms) : IDisposable
{
    public void Dispose()
    {
        foreach (var waveform in Waveforms)
        {
            waveform.Dispose();
        }
    }
}
