using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed class OptionsDto
{
    [Key(0)]
    public double TimeTolerance { get; init; }
    [Key(1)]
    public double AmpTolerance { get; init; }
    [Key(2)]
    public double PhaseTolerance { get; init; }
    [Key(3)]
    public bool AllowOversize { get; init; }


    private PulseGenOptions? _options;
    public PulseGenOptions GetOptions()
    {
        return _options ??= new()
        {
            TimeTolerance = TimeTolerance,
            AmpTolerance = AmpTolerance,
            PhaseTolerance = PhaseTolerance,
            AllowOversize = AllowOversize,
        };
    }
}
