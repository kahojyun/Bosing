using MessagePack;

namespace Qynit.PulseGen.Aot.Models;

[MessagePackObject]
public sealed record HannShapeInfo : ShapeInfo
{
    private static readonly IPulseShape PulseShape = new HannPulseShape();
    public override IPulseShape GetPulseShape()
    {
        return PulseShape;
    }
}
