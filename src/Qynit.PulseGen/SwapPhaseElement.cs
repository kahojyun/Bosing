namespace Qynit.PulseGen;
public sealed class SwapPhaseElement : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= new HashSet<int> { ChannelId1, ChannelId2 };

    public int ChannelId1 { get; }
    public int ChannelId2 { get; }

    public SwapPhaseElement(int channelId1, int channelId2)
    {
        ChannelId1 = channelId1;
        ChannelId2 = channelId2;
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        return 0;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        return 0;
    }

    protected override void RenderOverride(double time, PhaseTrackingTransform phaseTrackingTransform)
    {
        phaseTrackingTransform.SwapPhase(ChannelId1, ChannelId2, time);
    }
}
