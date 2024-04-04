using MessagePack;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class OptionsDto
{
    [Key(0)]
    public double TimeTolerance { get; set; }
    [Key(1)]
    public double AmpTolerance { get; set; }
    [Key(2)]
    public double PhaseTolerance { get; set; }
    [Key(3)]
    public bool AllowOversize { get; set; }


    private BosingOptions? _options;
    public BosingOptions GetOptions()
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
