using CommunityToolkit.Diagnostics;

using MessagePack;

using Bosing.Schedules;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class StackScheduleDto : ScheduleElementDto
{
    [Key(6)]
    public IList<ScheduleElementDto>? Elements { get; set; }
    [Key(7)]
    public ArrangeOption ArrangeOption { get; set; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(Elements);
        var result = new StackSchedule()
        {
            ArrangeOption = ArrangeOption,
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
            BosingOptions = request.Options?.GetOptions(),
        };
        foreach (var element in Elements)
        {
            result.Add(element.GetScheduleElement(request));
        }
        return result;
    }
}
