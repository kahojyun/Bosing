using MessagePack;

using Bosing.Schedules;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class ShiftPhaseElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId { get; set; }
    [Key(7)]
    public double DeltaPhase { get; set; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        return new ShiftPhaseElement(ChannelId, DeltaPhase)
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
