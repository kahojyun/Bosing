namespace Qynit.PulseGen;
public sealed class ShiftPhaseElement : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= new HashSet<int> { ChannelId };

    public int ChannelId { get; }
    public double DeltaPhase { get; }

    public ShiftPhaseElement(int channelId, double deltaPhase)
    {
        ChannelId = channelId;
        DeltaPhase = deltaPhase;
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
        phaseTrackingTransform.ShiftPhase(ChannelId, DeltaPhase);
    }
}
