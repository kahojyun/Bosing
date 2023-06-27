using System.Diagnostics;
using System.Runtime.CompilerServices;

namespace Qynit.Pulsewave;
public readonly ref struct ComplexArrayReadOnlySpan<T>
    where T : unmanaged
{
    public ReadOnlySpan<T> DataI { get; }
    public ReadOnlySpan<T> DataQ { get; }
    public int Length => DataI.Length;
    public bool IsEmpty => Length == 0;
    internal ComplexArrayReadOnlySpan(ReadOnlySpan<T> dataI, ReadOnlySpan<T> dataQ)
    {
        Debug.Assert(dataI.Length == dataQ.Length);
        DataI = dataI;
        DataQ = dataQ;
    }
    public static implicit operator ComplexArrayReadOnlySpan<T>(PooledComplexArray<T> source)
    {
        return new ComplexArrayReadOnlySpan<T>(source.DataI, source.DataQ);
    }
    public static implicit operator ComplexArrayReadOnlySpan<T>(ComplexArraySpan<T> source)
    {
        return new ComplexArrayReadOnlySpan<T>(source.DataI, source.DataQ);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ComplexArrayReadOnlySpan<T> Slice(int start)
    {
        return new ComplexArrayReadOnlySpan<T>(DataI[start..], DataQ[start..]);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ComplexArrayReadOnlySpan<T> Slice(int start, int length)
    {
        return new ComplexArrayReadOnlySpan<T>(DataI.Slice(start, length), DataQ.Slice(start, length));
    }

    public void CopyTo(ComplexArraySpan<T> destination)
    {
        DataI.CopyTo(destination.DataI);
        DataQ.CopyTo(destination.DataQ);
    }
}
