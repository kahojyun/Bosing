using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Qynit.PulseGen;
public abstract class ScheduleElement
{
    public ScheduleElement? Parent { get; internal set; }
    public Thickness Margin { get; set; }
    public bool IsVisible { get; set; }
    public double? DesiredDuration { get; private set; }
    public double? ActualDuration { get; private set; }
    public double? ActualTime { get; private set; }
    public abstract IReadOnlySet<int> Channels { get; }
    internal bool IsMeasuring { get; private set; }
    public void Measure(double maxDuration)
    {
        Debug.Assert(maxDuration >= 0 || double.IsPositiveInfinity(maxDuration));
        if (IsMeasuring)
        {
            ThrowHelper.ThrowInvalidOperationException("Already measuring");
        }
        IsMeasuring = true;
        var margin = Margin.Total;
        Debug.Assert(double.IsFinite(margin));
        var availableDuration = Math.Max(maxDuration - margin, 0);
        var desiredDuration = MeasureOverride(availableDuration) + margin;
        Debug.Assert(double.IsFinite(desiredDuration));
        DesiredDuration = Math.Max(desiredDuration, 0);
        IsMeasuring = false;
    }
    protected abstract double MeasureOverride(double maxDuration);
    public void Arrange(double time, double finalDuration)
    {
        Debug.Assert(double.IsFinite(time));
        Debug.Assert(double.IsFinite(finalDuration) && finalDuration >= 0);
        if (DesiredDuration is null)
        {
            ThrowHelper.ThrowInvalidOperationException("Not measured");
        }
        if (finalDuration < DesiredDuration)
        {
            ThrowHelper.ThrowArgumentOutOfRangeException(nameof(finalDuration), finalDuration, "Final duration is less than desired duration");
        }
        var innerTime = time + Margin.Start;
        Debug.Assert(double.IsFinite(innerTime));
        var innerDuration = Math.Max(finalDuration - Margin.Total, 0);
        var actualDuration = ArrangeOverride(innerTime, innerDuration);
        Debug.Assert(double.IsFinite(actualDuration));
        ActualDuration = actualDuration;
        ActualTime = innerTime;
    }
    protected abstract double ArrangeOverride(double time, double finalDuration);
}
