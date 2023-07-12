using System.Diagnostics;

using CommunityToolkit.Diagnostics;

namespace Qynit.PulseGen;
public class GridSchedule : ScheduleElement
{
    public override IReadOnlySet<int> Channels => _channels ??= _elements.SelectMany(e => e.Channels).ToHashSet();
    private HashSet<int>? _channels;
    private readonly List<ScheduleElement> _elements = new();

    public GridSchedule()
    {
        Alignment = Alignment.Stretch;
    }

    public void Add(ScheduleElement element)
    {
        if (element.Parent is not null)
        {
            ThrowHelper.ThrowArgumentException("The element is already added to another schedule.");
        }
        _elements.Add(element);
        element.Parent = this;
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        foreach (var element in _elements)
        {
            Debug.Assert(element.DesiredDuration is not null);
            var elementDuration = (element.Alignment == Alignment.Stretch) ? finalDuration : element.DesiredDuration.Value;
            Debug.Assert(elementDuration <= finalDuration);
            var elementTime = element.Alignment switch
            {
                Alignment.Start => 0,
                Alignment.Center => (finalDuration - elementDuration) / 2,
                Alignment.End => finalDuration - elementDuration,
                Alignment.Stretch => 0,
                _ => throw new NotImplementedException(),
            };
            element.Arrange(elementTime, elementDuration);
        }
        return finalDuration;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        foreach (var element in _elements)
        {
            element.Measure(maxDuration);
            Debug.Assert(element.DesiredDuration is not null);
        }
        return _elements.Count > 0 ? _elements.Max(e => e.DesiredDuration!.Value) : 0;
    }

    protected override void RenderOverride(double time, PhaseTrackingTransform phaseTrackingTransform)
    {
        foreach (var element in _elements)
        {
            element.Render(time, phaseTrackingTransform);
        }
    }
}
