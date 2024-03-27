using MessagePack;

using Qynit.PulseGen.Schedules;

namespace Qynit.PulseGen.Aot.Models;

[MessagePackObject]
public sealed class SetPhaseElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId { get; set; }
    [Key(7)]
    public double Phase { get; set; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        return new SetPhaseElement(ChannelId, Phase)
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
            PulseGenOptions = request.Options?.GetOptions(),
        };
    }
}
