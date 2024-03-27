using CommunityToolkit.Diagnostics;

using MessagePack;

using Qynit.PulseGen.Schedules;

namespace Qynit.PulseGen.Aot.Models;

[MessagePackObject]
public sealed class BarrierElementDto : ScheduleElementDto
{
    [Key(6)]
    public ISet<int>? ChannelIds { get; set; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(ChannelIds);
        return new BarrierElement(ChannelIds)
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
