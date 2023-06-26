using System.Numerics;
using System.Runtime.CompilerServices;

namespace Qynit.Pulsewave;
public record struct IqPair<T>
    where T : unmanaged, INumber<T>, ITrigonometricFunctions<T>
{
    public static readonly IqPair<T> Zero = new(T.Zero, T.Zero);
    public static readonly IqPair<T> One = new(T.One, T.Zero);

    public T I { get; set; }
    public T Q { get; set; }

    public IqPair(T i, T q)
    {
        I = i;
        Q = q;
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> FromPolarCoordinates(T magnitude, T phase)
    {
        return new IqPair<T>(magnitude * T.Cos(phase), magnitude * T.Sin(phase));
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator +(IqPair<T> left, IqPair<T> right)
    {
        return new IqPair<T>(left.I + right.I, left.Q + right.Q);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator +(IqPair<T> left, T right)
    {
        return new IqPair<T>(left.I + right, left.Q);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator +(T left, IqPair<T> right)
    {
        return new IqPair<T>(left + right.I, right.Q);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator -(IqPair<T> left, IqPair<T> right)
    {
        return new IqPair<T>(left.I - right.I, left.Q - right.Q);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator -(IqPair<T> left, T right)
    {
        return new IqPair<T>(left.I - right, left.Q);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator -(T left, IqPair<T> right)
    {
        return new IqPair<T>(left - right.I, right.Q);
    }


    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator *(IqPair<T> left, IqPair<T> right)
    {
        return new IqPair<T>(left.I * right.I - left.Q * right.Q, left.I * right.Q + left.Q * right.I);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator *(IqPair<T> left, T right)
    {
        return new IqPair<T>(left.I * right, left.Q * right);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator *(T left, IqPair<T> right)
    {
        return new IqPair<T>(left * right.I, left * right.Q);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> Conjugate(IqPair<T> value)
    {
        return new IqPair<T>(value.I, -value.Q);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static implicit operator IqPair<T>(T value)
    {
        return new IqPair<T>(value, T.Zero);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public readonly void Deconstruct(out T i, out T q)
    {
        i = I;
        q = Q;
    }
}
