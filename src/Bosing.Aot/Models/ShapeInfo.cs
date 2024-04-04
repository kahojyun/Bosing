using MessagePack;

namespace Bosing.Aot.Models;

[Union(0, typeof(HannShapeInfo))]
[Union(1, typeof(TriangleShapeInfo))]
[Union(2, typeof(InterpolatedShapeInfo))]
[MessagePackObject]
public abstract record ShapeInfo
{
    public abstract IPulseShape GetPulseShape();
}
