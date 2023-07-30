using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed class PulseGenResponse : IDisposable
{
    [Key(0)]
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Usage", "MsgPack003:Use MessagePackObjectAttribute", Justification = "<Pending>")]
    public IList<PooledComplexArray> Waveforms { get; set; } = null!;

    private readonly List<ArcUnsafe<PooledComplexArray<float>>> _disposables = null!;

    public PulseGenResponse() { }

    public PulseGenResponse(List<ArcUnsafe<PooledComplexArray<float>>> waveforms)
    {
        _disposables = waveforms;
        Waveforms = _disposables.Select(x => (PooledComplexArray)x.Target).ToList();
    }

    public void Dispose()
    {
        foreach (var disposable in _disposables)
        {
            disposable.Dispose();
        }
    }
}
