using System.Buffers;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

using CommunityToolkit.Diagnostics;

namespace Qynit.PulseGen;

public abstract class PooledComplexArray { }

public unsafe sealed class PooledComplexArray<T> : PooledComplexArray, IDisposable
    where T : unmanaged
{
    public int Length { get; }
    public bool IsEmpty => Length == 0;
    public bool IsReal { get; set; }
    public Span<T> DataI
    {
        get
        {
            if (_dataI is null)
            {
                ThrowHelper.ThrowObjectDisposedException(nameof(PooledComplexArray<T>));
            }
            return new Span<T>(_dataI, Length);
        }
    }
    public Span<T> DataQ
    {
        get
        {
            if (_dataQ is null)
            {
                ThrowHelper.ThrowObjectDisposedException(nameof(PooledComplexArray<T>));
            }
            return new Span<T>(_dataQ, Length);
        }
    }
    private void* _dataI;
    private void* _dataQ;
    private bool _disposedValue;

    public PooledComplexArray(int length, bool clear)
    {
        if (length < 0)
        {
            ThrowHelper.ThrowArgumentOutOfRangeException(nameof(length));
        }
        Length = length;
        _dataI = NativeMemory.AlignedAlloc((nuint)length * (nuint)Unsafe.SizeOf<T>(), 256 / 8);
        _dataQ = NativeMemory.AlignedAlloc((nuint)length * (nuint)Unsafe.SizeOf<T>(), 256 / 8);
        if (clear)
        {
            Clear();
        }
    }
    public PooledComplexArray(ComplexReadOnlySpan<T> source) : this(source.Length, false)
    {
        source.DataI.CopyTo(DataI);
        source.DataQ.CopyTo(DataQ);
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

    private void Dispose(bool disposing)
    {
        if (!_disposedValue)
        {
            if (disposing)
            {
                // TODO: dispose managed state (managed objects)
            }

            // TODO: free unmanaged resources (unmanaged objects) and override finalizer
            // TODO: set large fields to null
            _disposedValue = true;
            if (_dataI is not null)
            {
                NativeMemory.AlignedFree(_dataI);
                _dataI = null;
            }
            if (_dataQ is not null)
            {
                NativeMemory.AlignedFree(_dataQ);
                _dataQ = null;
            }
        }
    }

    // TODO: override finalizer only if 'Dispose(bool disposing)' has code to free unmanaged resources
    ~PooledComplexArray()
    {
        // Do not change this code. Put cleanup code in 'Dispose(bool disposing)' method
        Dispose(disposing: false);
    }

    public void Dispose()
    {
        // Do not change this code. Put cleanup code in 'Dispose(bool disposing)' method
        Dispose(disposing: true);
        GC.SuppressFinalize(this);
    }
}
