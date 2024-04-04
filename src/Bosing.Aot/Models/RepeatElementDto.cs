using CommunityToolkit.Diagnostics;

using MessagePack;

using Bosing.Schedules;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class RepeatElementDto : ScheduleElementDto
{
    [Key(6)]
    public ScheduleElementDto? Element { get; set; }
    [Key(7)]
    public int Count { get; set; }
    [Key(8)]
    public double Spacing { get; set; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(Element);
        var element = Element.GetScheduleElement(request);
        return new RepeatElement(element, Count)
        {
            Spacing = Spacing,
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
