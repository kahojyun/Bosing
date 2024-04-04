using MessagePack;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed record HannShapeInfo : ShapeInfo
{
    private static readonly IPulseShape PulseShape = new HannPulseShape();
    public override IPulseShape GetPulseShape()
    {
        return PulseShape;
    }
}
