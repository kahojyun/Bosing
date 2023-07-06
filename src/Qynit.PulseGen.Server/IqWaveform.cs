using System.Runtime.InteropServices;

using MessagePack;

namespace Qynit.PulseGen.Server;

[MessagePackObject]
public sealed class IqWaveform : IDisposable
{
    [Key(0)]
    public int Length => _array.Length;
    [Key(1)]
    public Span<byte> DataI => MemoryMarshal.AsBytes(_array.DataI);
    [Key(2)]
    public Span<byte> DataQ => MemoryMarshal.AsBytes(_array.DataQ);

    private readonly PooledComplexArray<double> _array;

    public IqWaveform(PooledComplexArray<double> array)
    {
        _array = array;
    }

    public void Dispose()
    {
        _array.Dispose();
    }
}
