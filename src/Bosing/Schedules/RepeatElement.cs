using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Bosing.Schedules;
public class RepeatElement : ScheduleElement
{
    public override IReadOnlySet<int> Channels => ScheduleElement.Channels;

    public double Spacing { get; set; }
    public ScheduleElement ScheduleElement { get; }
    public int Count { get; }

    public RepeatElement(ScheduleElement scheduleElement, int count)
    {
        Guard.IsGreaterThanOrEqualTo(count, 0);
        if (scheduleElement.Parent is not null)
        {
            ThrowHelper.ThrowArgumentException("The element is already added to another schedule.");
        }
        scheduleElement.Parent = this;
        ScheduleElement = scheduleElement;
        Count = count;
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        var n = Count;
        if (n == 0)
        {
            return 0;
        }
        var spacing = Spacing;
        var durationPerRepeat = (finalDuration - spacing * (n - 1)) / n;
        ScheduleElement.Arrange(0, durationPerRepeat);
        return finalDuration;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        var n = Count;
        if (n == 0)
        {
            return 0;
        }
        var spacing = Spacing;
        var durationPerRepeat = (maxDuration - spacing * (n - 1)) / n;
        ScheduleElement.Measure(durationPerRepeat);
        Debug.Assert(ScheduleElement.DesiredDuration is not null);
        return ScheduleElement.DesiredDuration.Value * n + spacing * (n - 1);
    }

    protected override void RenderOverride(double time, PhaseTrackingTransform phaseTrackingTransform)
    {
        var n = Count;
        if (n == 0)
        {
            return;
        }
        var spacing = Spacing;
        Debug.Assert(ActualDuration is not null);
        var durationPerRepeat = (ActualDuration.Value - spacing * (n - 1)) / n;
        for (var i = 0; i < n; i++)
        {
            var innerTime = time + i * (durationPerRepeat + spacing);
            ScheduleElement.Render(innerTime, phaseTrackingTransform);
        }
    }
}
