using System.Numerics;
using System.Runtime.CompilerServices;

namespace Bosing;
public readonly record struct IqPair<T>(T I, T Q)
    where T : unmanaged, INumber<T>, ITrigonometricFunctions<T>
{
    public static readonly IqPair<T> Zero = new(T.Zero, T.Zero);
    public static readonly IqPair<T> One = new(T.One, T.Zero);
    public static readonly IqPair<T> ImaginaryOne = new(T.Zero, T.One);

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
    public static IqPair<T> operator +(IqPair<T> value)
    {
        return value;
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
        return new IqPair<T>(left - right.I, -right.Q);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator -(IqPair<T> value)
    {
        return new IqPair<T>(-value.I, -value.Q);
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
    public static IqPair<T> operator /(IqPair<T> left, IqPair<T> right)
    {
        // BCL implementation
        var a = left.I;
        var b = left.Q;
        var c = right.I;
        var d = right.Q;

        if (T.Abs(d) < T.Abs(c))
        {
            var doc = d / c;
            return new IqPair<T>((a + b * doc) / (c + d * doc), (b - a * doc) / (c + d * doc));
        }
        else
        {
            var cod = c / d;
            return new IqPair<T>((b + a * cod) / (d + c * cod), (-a + b * cod) / (d + c * cod));
        }
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator /(IqPair<T> left, T right)
    {
        return new IqPair<T>(left.I / right, left.Q / right);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IqPair<T> operator /(T left, IqPair<T> right)
    {
        // BCL implementation
        var a = left;
        var c = right.I;
        var d = right.Q;

        if (T.Abs(d) < T.Abs(c))
        {
            var doc = d / c;
            return new IqPair<T>(a / (c + d * doc), -a * doc / (c + d * doc));
        }
        else
        {
            var cod = c / d;
            return new IqPair<T>(a * cod / (d + c * cod), -a / (d + c * cod));
        }
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
    public static explicit operator IqPair<T>(Complex value)
    {
        return new IqPair<T>(T.CreateChecked(value.Real), T.CreateChecked(value.Imaginary));
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public readonly void Deconstruct(out T i, out T q)
    {
        i = I;
        q = Q;
    }
}
