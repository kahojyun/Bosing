using System.Buffers;

using CommunityToolkit.Diagnostics;

namespace Qynit.Pulsewave;
public sealed class PooledComplexArray<T> : IDisposable
    where T : unmanaged
{
    public int Length { get; }
    public Span<T> DataI
    {
        get
        {
            if (_disposed)
            {
                ThrowHelper.ThrowObjectDisposedException(nameof(PooledComplexArray<T>));
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
                ThrowHelper.ThrowObjectDisposedException(nameof(PooledComplexArray<T>));
            }
            return _dataQ.AsSpan(0, Length);
        }
    }
    private readonly T[] _dataI;
    private readonly T[] _dataQ;
    private bool _disposed;

    public PooledComplexArray(int length, bool clear)
    {
        Length = length;
        _dataI = ArrayPool<T>.Shared.Rent(length);
        _dataQ = ArrayPool<T>.Shared.Rent(length);
        if (clear)
        {
            Clear();
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
    public void CopyTo(ComplexArraySpan<T> destination)
    {
        DataI.CopyTo(destination.DataI);
        DataQ.CopyTo(destination.DataQ);
    }
    public ComplexArraySpan<T> AsSpan()
    {
        return new ComplexArraySpan<T>(DataI, DataQ);
    }
    public ComplexArraySpan<T> AsSpan(int start)
    {
        return new ComplexArraySpan<T>(DataI[start..], DataQ[start..]);
    }
    public ComplexArraySpan<T> AsSpan(Index start)
    {
        return new ComplexArraySpan<T>(DataI[start..], DataQ[start..]);
    }
    public ComplexArraySpan<T> AsSpan(int start, int length)
    {
        return new ComplexArraySpan<T>(DataI.Slice(start, length), DataQ.Slice(start, length));
    }
    public ComplexArraySpan<T> AsSpan(Range range)
    {
        return new ComplexArraySpan<T>(DataI[range], DataQ[range]);
    }
    public void Clear()
    {
        _dataI.AsSpan(0, Length).Clear();
        _dataQ.AsSpan(0, Length).Clear();
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
}
