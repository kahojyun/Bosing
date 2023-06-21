using System.Numerics;

namespace Qynit.Pulsewave;
public class TrianglePulseShape : IPulseShape
{
    public Complex SampleAt(double x)
    {
        return (x >= -0.5 && x <= 0.5) ? 1 - 2 * Math.Abs(x) : 0;
    }
}
