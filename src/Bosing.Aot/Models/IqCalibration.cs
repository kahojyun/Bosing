using MessagePack;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class IqCalibration
{
    [Key(0)]
    public double A { get; set; }
    [Key(1)]
    public double B { get; set; }
    [Key(2)]
    public double C { get; set; }
    [Key(3)]
    public double D { get; set; }
    [Key(4)]
    public double IOffset { get; set; }
    [Key(5)]
    public double QOffset { get; set; }
}
