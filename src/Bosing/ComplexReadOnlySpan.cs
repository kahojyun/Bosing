using System.Diagnostics;
using System.Runtime.CompilerServices;

namespace Bosing;
public readonly ref struct ComplexReadOnlySpan<T>
    where T : unmanaged
{
    public static ComplexReadOnlySpan<T> Empty => new([], []);
    public ReadOnlySpan<T> DataI { get; }
    public ReadOnlySpan<T> DataQ { get; }
    public int Length => DataI.Length;
    public bool IsEmpty => Length == 0;
    internal ComplexReadOnlySpan(ReadOnlySpan<T> dataI, ReadOnlySpan<T> dataQ)
    {
        Debug.Assert(dataI.Length == dataQ.Length);
        DataI = dataI;
        DataQ = dataQ;
    }
    public static implicit operator ComplexReadOnlySpan<T>(PooledComplexArray<T> source)
    {
        return new ComplexReadOnlySpan<T>(source.DataI, source.DataQ);
    }
    public static implicit operator ComplexReadOnlySpan<T>(ComplexSpan<T> source)
    {
        return new ComplexReadOnlySpan<T>(source.DataI, source.DataQ);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ComplexReadOnlySpan<T> Slice(int start)
    {
        return new ComplexReadOnlySpan<T>(DataI[start..], DataQ[start..]);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ComplexReadOnlySpan<T> Slice(int start, int length)
    {
        return new ComplexReadOnlySpan<T>(DataI.Slice(start, length), DataQ.Slice(start, length));
    }

    public void CopyTo(ComplexSpan<T> destination)
    {
        DataI.CopyTo(destination.DataI);
        DataQ.CopyTo(destination.DataQ);
    }
}
