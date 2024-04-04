namespace Bosing;
internal class MathUtils
{
    /// <summary>
    /// Clamps a value between a minimum and maximum value.
    /// </summary>
    /// <remarks>
    /// If <paramref name="min"/> is greater than <paramref name="max"/>,
    /// <paramref name="min"/> is returned rather than throwing an exception as
    /// in <see cref="Math.Clamp(double, double, double)">standard
    /// library</see>.
    /// </remarks>
    public static double Clamp(double value, double min, double max)
    {
        return Math.Max(Math.Min(value, max), min);
    }

    public static bool IsApproximatelyEqual(double a, double b, double tolerance)
    {
        return Math.Abs(a - b) <= tolerance;
    }

    public static bool IsApproximatelyZero(double a, double tolerance)
    {
        return Math.Abs(a) <= tolerance;
    }

    public static bool IsApproximatelyEqualModulo(double a, double b, double modulo, double tolerance)
    {
        return IsApproximatelyZero(Math.IEEERemainder(a - b, modulo), tolerance);
    }

    public static bool IsApproximatelyZeroModulo(double a, double modulo, double tolerance)
    {
        return IsApproximatelyZero(Math.IEEERemainder(a, modulo), tolerance);
    }
}
