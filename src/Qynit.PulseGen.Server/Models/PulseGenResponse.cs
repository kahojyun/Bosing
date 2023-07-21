using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed class PulseGenResponse : IDisposable
{
    [Key(0)]
    public IList<PooledComplexArray<double>> Waveforms { get; set; } = null!;

    private readonly List<ArcUnsafe<PooledComplexArray<double>>> _disposables = null!;

    public PulseGenResponse() { }

    public PulseGenResponse(List<ArcUnsafe<PooledComplexArray<double>>> waveforms)
    {
        _disposables = waveforms;
        Waveforms = _disposables.Select(x => x.Target).ToList();
    }

    public void Dispose()
    {
        foreach (var disposable in _disposables)
        {
            disposable.Dispose();
        }
    }
}
