using System.Diagnostics;
using System.Runtime.CompilerServices;

namespace Bosing;
public readonly ref struct ComplexSpan<T>
    where T : unmanaged
{
    public static ComplexSpan<T> Empty => new([], []);
    public Span<T> DataI { get; }
    public Span<T> DataQ { get; }
    public int Length => DataI.Length;
    public bool IsEmpty => Length == 0;
    internal ComplexSpan(Span<T> dataI, Span<T> dataQ)
    {
        Debug.Assert(dataI.Length == dataQ.Length);
        DataI = dataI;
        DataQ = dataQ;
    }
    public static implicit operator ComplexSpan<T>(PooledComplexArray<T> source)
    {
        return new ComplexSpan<T>(source.DataI, source.DataQ);
    }

    public void Fill(T i, T q)
    {
        DataI.Fill(i);
        DataQ.Fill(q);
    }
    public void Clear()
    {
        DataI.Clear();
        DataQ.Clear();
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
}
