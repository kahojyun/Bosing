using System.Diagnostics;

namespace Qynit.Pulsewave;
internal sealed record EnvelopeSample<T> : IDisposable
    where T : unmanaged
{
    public PooledComplexArray<T>? LeftEdge { get; }
    public PooledComplexArray<T>? RightEdge { get; }
    public int Plateau { get; }
    public int Size => (LeftEdge?.Length ?? 0) + (RightEdge?.Length ?? 0);

    private EnvelopeSample(PooledComplexArray<T>? leftEdge, PooledComplexArray<T>? rightEdge, int plateau)
    {
        LeftEdge = leftEdge;
        RightEdge = rightEdge;
        Plateau = plateau;
    }

    public void Dispose()
    {
        LeftEdge?.Dispose();
        RightEdge?.Dispose();
    }

    public static EnvelopeSample<T>? Rectangle(int plateau)
    {
        return (plateau > 0) ? new EnvelopeSample<T>(null, null, plateau) : null;
    }

    public static EnvelopeSample<T> Continuous(PooledComplexArray<T> envelope)
    {
        return new EnvelopeSample<T>(envelope, null, 0);
    }

    public static EnvelopeSample<T> WithPlateau(PooledComplexArray<T> leftEdge, PooledComplexArray<T> rightEdge, int plateau)
    {
        Debug.Assert(plateau > 0);
        return new EnvelopeSample<T>(leftEdge, rightEdge, plateau);
    }
}
