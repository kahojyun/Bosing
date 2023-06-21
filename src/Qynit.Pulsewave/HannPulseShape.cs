using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Qynit.Pulsewave;
public sealed class HannPulseShape : IPulseShape
{
    public Complex SampleAt(double x)
    {
        return (x >= -0.5 && x <= 0.5) ? (1 + Math.Cos(Math.Tau * x)) / 2 : 0;
    }

    public void SampleIQ(Span<double> targetI, Span<double> targetQ, double x0, double dx)
    {
        var l = targetI.Length;
        Debug.Assert(targetQ.Length == l);

        var c = Complex.FromPolarCoordinates(0.5, Math.Tau * x0);
        var w = Complex.FromPolarCoordinates(1, Math.Tau * dx);

        ref var ti = ref MemoryMarshal.GetReference(targetI);
        ref var tq = ref MemoryMarshal.GetReference(targetQ);
        for (var i = 0; i < l; i++)
        {
            var x = x0 + i * dx;
            Unsafe.Add(ref ti, i) = (x >= -0.5 && x <= 0.5) ? 0.5 + c.Real : 0;
            Unsafe.Add(ref tq, i) = 0;
            c *= w;
        }
    }
}
