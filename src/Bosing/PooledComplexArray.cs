using System.Buffers;
using System.Runtime.CompilerServices;

using CommunityToolkit.Diagnostics;

namespace Bosing;

public abstract class PooledComplexArray { }

public sealed class PooledComplexArray<T> : PooledComplexArray, IDisposable
    where T : unmanaged
{
    public int Length { get; }
    public bool IsEmpty => Length == 0;
    public bool IsReal { get; set; }
    private static readonly ArrayPool<T> ArrayPool = ArrayPool<T>.Create(1000000, 1000);
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
        _dataI = ArrayPool.Rent(length);
        _dataQ = ArrayPool.Rent(length);
        if (clear)
        {
            Clear();
        }
    }
    public PooledComplexArray(ComplexReadOnlySpan<T> source) : this(source.Length, false)
    {
        source.DataI.CopyTo(_dataI);
        source.DataQ.CopyTo(_dataQ);
    }

    public PooledComplexArray<T> Copy()
    {
        return new PooledComplexArray<T>(this);
    }
    public void CopyTo(ComplexSpan<T> destination)
    {
        ((ComplexReadOnlySpan<T>)this).CopyTo(destination);
    }
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ComplexSpan<T> Slice(int start)
    {
        return new ComplexSpan<T>(DataI[start..], DataQ[start..]);
    }
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ComplexSpan<T> Slice(int start, int length)
    {
        return new ComplexSpan<T>(DataI.Slice(start, length), DataQ.Slice(start, length));
    }
    public void Clear()
    {
        DataI.Clear();
        DataQ.Clear();
    }
    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }
        _disposed = true;
        ArrayPool.Return(_dataI, false);
        ArrayPool.Return(_dataQ, false);
    }
}
