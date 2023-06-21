using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Qynit.Pulsewave;

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
    Complex SampleAt(double x);

    void SampleIQ(Span<double> targetI, Span<double> targetQ, double x0, double dx)
    {
        var l = targetI.Length;
        Debug.Assert(targetQ.Length == l);

        ref var ti = ref MemoryMarshal.GetReference(targetI);
        ref var tq = ref MemoryMarshal.GetReference(targetQ);
        for (var i = 0; i < l; i++)
        {
            var x = x0 + i * dx;
            var y = SampleAt(x);
            Unsafe.Add(ref ti, i) = y.Real;
            Unsafe.Add(ref tq, i) = y.Imaginary;
        }
    }
}
