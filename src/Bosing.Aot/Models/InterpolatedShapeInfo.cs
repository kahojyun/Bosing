using MessagePack;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed record InterpolatedShapeInfo(
    [property: Key(0)] double[] X,
    [property: Key(1)] double[] Y) : ShapeInfo
{
    private InterpolatedPulseShape? _pulseShape;
    public override IPulseShape GetPulseShape()
    {
        return _pulseShape ??= InterpolatedPulseShape.CreateFromXY(X, Y);
    }
}
