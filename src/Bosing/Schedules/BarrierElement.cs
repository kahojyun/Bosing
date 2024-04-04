namespace Bosing.Schedules;
public class BarrierElement(IEnumerable<int> channelIds) : ScheduleElement
{
    public override IReadOnlySet<int> Channels { get; } = new HashSet<int>(channelIds);

    public BarrierElement(params int[] channelIds) : this((IEnumerable<int>)channelIds) { }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        return 0;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        return 0;
    }

    protected override void RenderOverride(double time, PhaseTrackingTransform phaseTrackingTransform) { }
}
