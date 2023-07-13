using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Qynit.PulseGen;
public class AbsoluteSchedule : Schedule
{
    private readonly List<double> _elementTimes = new();
    public void Add(ScheduleElement element)
    {
        Add(element, 0);
    }

    public void Add(ScheduleElement element, double time)
    {
        if (element.Parent is not null)
        {
            ThrowHelper.ThrowArgumentException("The element is already added to another schedule.");
        }
        if (!double.IsFinite(time))
        {
            ThrowHelper.ThrowArgumentException("The time is not finite.");
        }
        Children.Add(element);
        element.Parent = this;
        _elementTimes.Add(time);
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        foreach (var (element, elementTime) in Children.Zip(_elementTimes))
        {
            Debug.Assert(element.DesiredDuration is not null);
            element.Arrange(elementTime, element.DesiredDuration.Value);
        }
        return finalDuration;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        var maxTime = 0.0;
        foreach (var (element, time) in Children.Zip(_elementTimes))
        {
            element.Measure(maxDuration);
            Debug.Assert(element.DesiredDuration is not null);
            maxTime = Math.Max(maxTime, time + element.DesiredDuration.Value);
        }
        return maxTime;
    }
}
