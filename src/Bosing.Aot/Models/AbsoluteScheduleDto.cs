using CommunityToolkit.Diagnostics;

using MessagePack;

using Bosing.Schedules;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class AbsoluteScheduleDto : ScheduleElementDto
{
    [Key(6)]
    public IList<(double Time, ScheduleElementDto Element)>? Elements { get; set; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(Elements);
        var result = new AbsoluteSchedule()
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
            BosingOptions = request.Options?.GetOptions(),
        };
        foreach (var (time, element) in Elements)
        {
            result.Add(element.GetScheduleElement(request), time);
        }
        return result;
    }
}
