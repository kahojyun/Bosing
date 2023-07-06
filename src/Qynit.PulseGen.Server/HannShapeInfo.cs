using MessagePack;

namespace Qynit.PulseGen.Server;

[MessagePackObject]
public sealed record HannShapeInfo : ShapeInfo
{
    private static readonly IPulseShape PulseShape = new HannPulseShape();
    public override IPulseShape GetPulseShape()
    {
        return PulseShape;
    }
}
