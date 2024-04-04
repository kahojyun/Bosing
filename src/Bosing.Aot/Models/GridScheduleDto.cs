using CommunityToolkit.Diagnostics;

using MessagePack;

using Bosing.Schedules;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class GridScheduleDto : ScheduleElementDto
{
    [Key(6)]
    public IList<(int Column, int Span, ScheduleElementDto Element)>? Elements { get; set; }
    [Key(7)]
    public IList<(double Value, GridLengthUnit Unit)>? Columns { get; set; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(Elements);
        Guard.IsNotNull(Columns);
        var result = new GridSchedule()
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
            BosingOptions = request.Options?.GetOptions(),
        };
        foreach (var (column, span, element) in Elements)
        {
            result.Add(element.GetScheduleElement(request), column, span);
        }
        foreach (var (value, unit) in Columns)
        {
            result.AddColumn(new(value, unit));
        }
        return result;
    }
}
