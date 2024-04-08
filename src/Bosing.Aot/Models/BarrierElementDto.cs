using Bosing.Schedules;

using CommunityToolkit.Diagnostics;

using MessagePack;

namespace Bosing.Aot.Models;

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
            BosingOptions = request.Options?.GetOptions(),
        };
    }
}
