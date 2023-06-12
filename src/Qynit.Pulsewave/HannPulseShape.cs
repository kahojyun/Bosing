using System.Numerics;

namespace Qynit.Pulsewave;
public sealed class HannPulseShape : IPulseShape
{
    public Complex SampleAt(double x)
    {
        return (x >= -0.5 && x <= 0.5) ? (1 + Math.Cos(Math.Tau * x)) / 2 : 0;
    }
}
