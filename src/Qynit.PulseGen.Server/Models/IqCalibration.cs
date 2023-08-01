using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed class IqCalibration
{
    [Key(0)]
    public double A { get; init; }
    [Key(1)]
    public double B { get; init; }
    [Key(2)]
    public double C { get; init; }
    [Key(3)]
    public double D { get; init; }
    [Key(4)]
    public double IOffset { get; init; }
    [Key(5)]
    public double QOffset { get; init; }
}
