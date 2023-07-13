namespace Qynit.PulseGen;
public class BarrierElement : ScheduleElement
{
    public override IReadOnlySet<int> Channels { get; }

    public BarrierElement(params int[] channelIds) : this((IEnumerable<int>)channelIds) { }

    public BarrierElement(IEnumerable<int> channelIds)
    {
        Channels = new HashSet<int>(channelIds);
    }

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
