using System.Diagnostics;

namespace Bosing;
public sealed class EnvelopeSample<T>
    where T : unmanaged
{
    public ComplexReadOnlySpan<T> LeftEdge => _leftArray ?? ComplexReadOnlySpan<T>.Empty;
    public ComplexReadOnlySpan<T> RightEdge => _rightArray ?? ComplexReadOnlySpan<T>.Empty;
    public int Size => (_leftArray?.Length ?? 0) + (_rightArray?.Length ?? 0);
    public int Plateau { get; }
    private readonly PooledComplexArray<T>? _leftArray;
    private readonly PooledComplexArray<T>? _rightArray;

    private EnvelopeSample(PooledComplexArray<T>? leftEdge, PooledComplexArray<T>? rightEdge, int plateau)
    {
        _leftArray = leftEdge;
        _rightArray = rightEdge;
        Plateau = plateau;
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
