using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Qynit.PulseGen;
public class StackSchedule : Schedule
{
    public ArrangeOption ArrangeOption { get; set; }

    public StackSchedule()
    {
        Alignment = Alignment.Stretch;
    }

    public void Add(ScheduleElement element)
    {
        if (element.Parent is not null)
        {
            ThrowHelper.ThrowArgumentException("The element is already added to another schedule.");
        }
        Children.Add(element);
        element.Parent = this;
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        var channels = Channels;
        var arrangeOption = ArrangeOption;
        var elements = arrangeOption switch
        {
            ArrangeOption.StartToEnd => Children.AsEnumerable(),
            ArrangeOption.EndToStart => Children.AsEnumerable().Reverse(),
            _ => throw new NotImplementedException(),
        };
        var durations = channels.ToDictionary(c => c, _ => 0.0);
        foreach (var element in elements)
        {
            var elementChannels = element.Channels;
            Debug.Assert(element.DesiredDuration is not null);
            var innerDuration = element.DesiredDuration.Value;
            var usedDuration = elementChannels.Max(c => durations[c]);
            var innerTime = arrangeOption switch
            {
                ArrangeOption.StartToEnd => usedDuration,
                ArrangeOption.EndToStart => finalDuration - usedDuration - innerDuration,
                _ => throw new NotImplementedException(),
            };
            element.Arrange(innerTime, innerDuration);
            var newDuration = usedDuration + innerDuration;
            Debug.Assert(double.IsFinite(newDuration));
            foreach (var channel in elementChannels)
            {
                durations[channel] = newDuration;
            }
        }
        return finalDuration;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        var channels = Channels;
        var elements = ArrangeOption switch
        {
            ArrangeOption.StartToEnd => Children.AsEnumerable(),
            ArrangeOption.EndToStart => Children.AsEnumerable().Reverse(),
            _ => throw new NotImplementedException(),
        };
        var durations = channels.ToDictionary(c => c, _ => 0.0);
        foreach (var element in elements)
        {
            var elementChannels = element.Channels;
            var usedDuration = elementChannels.Max(c => durations[c]);
            var leftDuration = maxDuration - usedDuration;
            element.Measure(leftDuration);
            var desiredDuration = element.DesiredDuration;
            Debug.Assert(desiredDuration is not null);
            var newDuration = usedDuration + desiredDuration.Value;
            Debug.Assert(double.IsFinite(newDuration));
            foreach (var channel in elementChannels)
            {
                durations[channel] = newDuration;
            }
        }
        return durations.Values.Max();
    }
}
