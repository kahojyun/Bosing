using MessagePack;

namespace Qynit.PulseGen.Server;

[Union(0, typeof(HannShapeInfo))]
[Union(1, typeof(TriangleShapeInfo))]
[Union(2, typeof(InterpolatedShapeInfo))]
public abstract record ShapeInfo
{
    public abstract IPulseShape GetPulseShape();
}
