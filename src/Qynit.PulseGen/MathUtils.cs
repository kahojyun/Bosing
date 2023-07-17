namespace Qynit.PulseGen;
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
}
