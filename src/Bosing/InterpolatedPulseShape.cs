using System.Numerics;

using BitFaster.Caching.Lru;

using CommunityToolkit.Diagnostics;

using MathNet.Numerics;
using MathNet.Numerics.Interpolation;

namespace Bosing;

public sealed record InterpolatedPulseShape(IInterpolation Interpolation) : IPulseShape
{
    public IqPair<T> SampleAt<T>(T x) where T : unmanaged, IFloatingPointIeee754<T>
    {
        var half = T.CreateChecked(0.5);
        return (x >= -half && x <= half)
            ? T.CreateChecked(Interpolation.Interpolate(double.CreateChecked(x)))
            : T.Zero;
    }

    public static InterpolatedPulseShape CreateFromXY(IReadOnlyList<double> x, IReadOnlyList<double> y)
    {
        if (x.Count != y.Count)
        {
            ThrowHelper.ThrowArgumentException("x and y must have the same length.");
        }
        var key = (new ValueArray<double>(x), new ValueArray<double>(y));
        return Cache.GetOrAdd(key, k =>
        {
            var interpolation = Interpolate.RationalWithoutPoles(x, y);
            return new InterpolatedPulseShape(interpolation);
        });
    }


    private static readonly FastConcurrentLru<(ValueArray<double>, ValueArray<double>), InterpolatedPulseShape> Cache = new(666);
}
