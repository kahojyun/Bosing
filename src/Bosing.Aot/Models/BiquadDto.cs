using MessagePack;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class BiquadDto
{
    [Key(0)]
    public double B0 { get; set; }
    [Key(1)]
    public double B1 { get; set; }
    [Key(2)]
    public double B2 { get; set; }
    [Key(3)]
    public double A1 { get; set; }
    [Key(4)]
    public double A2 { get; set; }

    public BiquadCoefficients<double> GetBiquad()
    {
        return new BiquadCoefficients<double>
        {
            B0 = B0,
            B1 = B1,
            B2 = B2,
            A1 = A1,
            A2 = A2,
        };
    }
}
