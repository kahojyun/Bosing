namespace Bosing.Schedules;
public sealed class SwapPhaseElement(int channelId1, int channelId2) : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= [ChannelId1, ChannelId2];

    public int ChannelId1 { get; } = channelId1;
    public int ChannelId2 { get; } = channelId2;

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
