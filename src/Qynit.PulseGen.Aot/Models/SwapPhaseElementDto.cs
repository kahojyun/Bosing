using MessagePack;

using Qynit.PulseGen.Schedules;

namespace Qynit.PulseGen.Aot.Models;

[MessagePackObject]
public sealed class SwapPhaseElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId1 { get; set; }
    [Key(7)]
    public int ChannelId2 { get; set; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        return new SwapPhaseElement(ChannelId1, ChannelId2)
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
