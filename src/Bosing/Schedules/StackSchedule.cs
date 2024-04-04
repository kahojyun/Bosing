using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Bosing.Schedules;
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
        var helper = new LayoutHelper(Channels, ArrangeOption, Children);
        foreach (var element in helper.ChildrenEnumerable)
        {
            var elementChannels = element.Channels;
            var usedDuration = helper.GetUsed(elementChannels);
            Debug.Assert(element.DesiredDuration is not null);
            var innerDuration = element.DesiredDuration.Value;
            var innerTime = helper.GetArrangeTime(usedDuration, innerDuration, finalDuration);
            element.Arrange(innerTime, innerDuration);
            var newDuration = usedDuration + innerDuration;
            Debug.Assert(double.IsFinite(newDuration));
            helper.UpdateUsed(elementChannels, newDuration);
        }
        return finalDuration;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        var helper = new LayoutHelper(Channels, ArrangeOption, Children);
        foreach (var element in helper.ChildrenEnumerable)
        {
            var elementChannels = element.Channels;
            var usedDuration = helper.GetUsed(elementChannels);
            var leftDuration = maxDuration - usedDuration;
            element.Measure(leftDuration);
            Debug.Assert(element.DesiredDuration is not null);
            var newDuration = usedDuration + element.DesiredDuration.Value;
            Debug.Assert(double.IsFinite(newDuration));
            helper.UpdateUsed(elementChannels, newDuration);
        }
        return helper.GetTotalUsed();
    }

    private struct LayoutHelper
    {
        private readonly IReadOnlySet<int> _channels;
        private readonly Dictionary<int, double>? _channelDurations;
        private readonly ArrangeOption _arrangeOption;
        private readonly List<ScheduleElement> _children;
        private double _usedDuration;

        public LayoutHelper(IReadOnlySet<int> channels, ArrangeOption arrangeOption, List<ScheduleElement> children)
        {
            _channels = channels;
            if (channels.Count > 0)
            {
                _channelDurations = channels.ToDictionary(c => c, _ => 0.0);
            }
            _arrangeOption = arrangeOption;
            _children = children;
        }

        public readonly IEnumerable<ScheduleElement> ChildrenEnumerable => _arrangeOption switch
        {
            ArrangeOption.StartToEnd => _children.AsEnumerable(),
            ArrangeOption.EndToStart => _children.AsEnumerable().Reverse(),
            _ => throw new NotImplementedException(),
        };

        public readonly double GetUsed(IReadOnlySet<int> channels)
        {
            var channelDurations = _channelDurations;
            return channelDurations is null
                ? _usedDuration
                : channels.Count == 0 ? channelDurations.Values.Max() : channels.Max(c => channelDurations[c]);
        }

        public readonly double GetTotalUsed()
        {
            return _channelDurations?.Values.Max() ?? _usedDuration;
        }

        public readonly double GetArrangeTime(double usedDuration, double childDuration, double totalDuration)
        {
            return _arrangeOption switch
            {
                ArrangeOption.StartToEnd => usedDuration,
                ArrangeOption.EndToStart => totalDuration - usedDuration - childDuration,
                _ => throw new NotImplementedException(),
            };
        }

        public void UpdateUsed(IReadOnlySet<int> channels, double newValue)
        {
            if (_channelDurations is null)
            {
                _usedDuration = newValue;
                return;
            }
            foreach (var channel in channels.Count == 0 ? _channels : channels)
            {
                _channelDurations[channel] = newValue;
            }
        }
    }
}
