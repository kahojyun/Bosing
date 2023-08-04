namespace Qynit.PulseGen.Schedules;
public sealed class SetPhaseElement : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= new HashSet<int> { ChannelId };
    public int ChannelId { get; }
    public double Phase { get; }

    public SetPhaseElement(int channelId, double phase)
    {
        ChannelId = channelId;
        Phase = phase;
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
        phaseTrackingTransform.SetPhase(ChannelId, Phase, time);
    }
}
