﻿using MessagePack;

namespace Qynit.PulseGen.Server;

[MessagePackObject]
public sealed record TriangleShapeInfo : ShapeInfo
{
    private static readonly IPulseShape PulseShape = new TrianglePulseShape();
    public override IPulseShape GetPulseShape()
    {
        return PulseShape;
    }
}
