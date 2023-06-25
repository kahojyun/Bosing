using System.Diagnostics;
using System.Runtime.CompilerServices;

namespace Qynit.Pulsewave;
internal readonly ref struct ComplexArraySpan<T>
    where T : struct
{
    public Span<T> DataI { get; }
    public Span<T> DataQ { get; }
    public int Length => DataI.Length;
    internal ComplexArraySpan(Span<T> dataI, Span<T> dataQ)
    {
        Debug.Assert(dataI.Length == dataQ.Length);
        DataI = dataI;
        DataQ = dataQ;
    }
    public static implicit operator ComplexArraySpan<T>(PooledComplexArray<T> source)
    {
        return new ComplexArraySpan<T>(source.DataI, source.DataQ);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ComplexArraySpan<T> Slice(int start)
    {
        return new ComplexArraySpan<T>(DataI[start..], DataQ[start..]);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ComplexArraySpan<T> Slice(int start, int length)
    {
        return new ComplexArraySpan<T>(DataI.Slice(start, length), DataQ.Slice(start, length));
    }
}
