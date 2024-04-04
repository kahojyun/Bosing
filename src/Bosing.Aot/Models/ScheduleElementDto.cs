using MessagePack;

using Bosing.Schedules;

namespace Bosing.Aot.Models;


[Union(0, typeof(PlayElementDto))]
[Union(1, typeof(ShiftPhaseElementDto))]
[Union(2, typeof(SetPhaseElementDto))]
[Union(3, typeof(ShiftFrequencyElementDto))]
[Union(4, typeof(SetFrequencyElementDto))]
[Union(5, typeof(SwapPhaseElementDto))]
[Union(6, typeof(BarrierElementDto))]
[Union(7, typeof(RepeatElementDto))]
[Union(8, typeof(StackScheduleDto))]
[Union(9, typeof(AbsoluteScheduleDto))]
[Union(10, typeof(GridScheduleDto))]
[MessagePackObject]
public abstract class ScheduleElementDto
{
    [Key(0)]
    public (double, double) MarginData { get; set; }
    [Key(1)]
    public Alignment Alignment { get; set; }
    [Key(2)]
    public bool IsVisible { get; set; }
    [Key(3)]
    public double? Duration { get; set; }
    [Key(4)]
    public double MaxDuration { get; set; }
    [Key(5)]
    public double MinDuration { get; set; }

    [IgnoreMember]
    public Thickness Margin => new(MarginData.Item1, MarginData.Item2);
    public abstract ScheduleElement GetScheduleElement(ScheduleRequest request);
}
