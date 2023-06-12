using System.Diagnostics.CodeAnalysis;

namespace Qynit.Pulsewave.Tests;
internal record ToleranceComparer(double Tolerance) : IEqualityComparer<double>
{
    public bool Equals(double x, double y)
    {
        return Math.Abs(x - y) <= Tolerance;
    }

    public int GetHashCode([DisallowNull] double obj)
    {
        throw new NotImplementedException();
    }
}
