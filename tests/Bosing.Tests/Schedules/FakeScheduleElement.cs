using Bosing.Schedules;

namespace Bosing.Tests.Schedules;

public class FakeScheduleElement(double measureResult, double arrangeResult, IEnumerable<int> channels) : ScheduleElement
{
    public override IReadOnlySet<int> Channels { get; } = channels.ToHashSet();

    public bool Flexible { get; init; }
    public Action<double>? RenderCallback { get; init; }

    public FakeScheduleElement() : this(0, 0)
    {
    }

    public FakeScheduleElement(double measureResult, double arrangeResult) : this(measureResult, arrangeResult, [])
    {
    }

    public FakeScheduleElement(IEnumerable<int> channels) : this(0, 0, channels)
    {
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        return Flexible ? finalDuration : arrangeResult;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        return measureResult;
    }

    protected override void RenderOverride(double time, PhaseTrackingTransform phaseTrackingTransform)
    {
        RenderCallback?.Invoke(time);
    }
}
