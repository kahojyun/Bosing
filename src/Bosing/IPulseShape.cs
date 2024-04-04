using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Bosing;

/// <summary>
/// Represents a pulse shape envelope.
/// </summary>
public interface IPulseShape
{
    /// <summary>
    /// Sample pulse shape envelope at x.
    /// </summary>
    /// <param name="x">x value in range -0.5 to -0.5</param>
    /// <returns>y value sampled at x</returns>
    IqPair<T> SampleAt<T>(T x)
        where T : unmanaged, IFloatingPointIeee754<T>;

    void SampleIQ<T>(ComplexSpan<T> target, T x0, T dx)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = target.Length;
        ref var ti = ref MemoryMarshal.GetReference(target.DataI);
        ref var tq = ref MemoryMarshal.GetReference(target.DataQ);
        var ii = T.Zero;
        for (var i = 0; i < length; i++, ii++)
        {
            var x = x0 + ii * dx;
            var (yi, yq) = SampleAt(x);
            Unsafe.Add(ref ti, i) = yi;
            Unsafe.Add(ref tq, i) = yq;
        }
    }
}
