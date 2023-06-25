using System.Buffers;

using CommunityToolkit.Diagnostics;

namespace Qynit.Pulsewave;
internal sealed class PooledComplexArray<T> : IDisposable
    where T : struct
{
    private readonly T[] _dataI;
    private readonly T[] _dataQ;
    private bool _disposed;
    public int Length { get; }
    public Span<T> DataI
    {
        get
        {
            if (_disposed)
            {
                ThrowHelper.ThrowObjectDisposedException(nameof(Waveform));
            }
            return _dataI.AsSpan(0, Length);
        }
    }
    public Span<T> DataQ
    {
        get
        {
            if (_disposed)
            {
                ThrowHelper.ThrowObjectDisposedException(nameof(Waveform));
            }
            return _dataQ.AsSpan(0, Length);
        }
    }

    public PooledComplexArray(int length, bool clear)
    {
        Length = length;
        _dataI = ArrayPool<T>.Shared.Rent(length);
        _dataQ = ArrayPool<T>.Shared.Rent(length);
        if (clear)
        {
            ClearData();
        }
    }

    public PooledComplexArray(ComplexArrayReadOnlySpan<T> source) : this(source.Length, false)
    {
        source.DataI.CopyTo(_dataI);
        source.DataQ.CopyTo(_dataQ);
    }

    public PooledComplexArray<T> Copy()
    {
        return new PooledComplexArray<T>(this);
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }

        _disposed = true;
        ArrayPool<T>.Shared.Return(_dataI, false);
        ArrayPool<T>.Shared.Return(_dataQ, false);
    }

    private void ClearData()
    {
        _dataI.AsSpan(0, Length).Clear();
        _dataQ.AsSpan(0, Length).Clear();
    }
}
