namespace Bosing.Schedules;
public sealed class ShiftFrequencyElement(int channelId, double deltaFrequency) : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= [ChannelId];

    public int ChannelId { get; } = channelId;
    public double DeltaFrequency { get; } = deltaFrequency;

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
        phaseTrackingTransform.ShiftFrequency(ChannelId, DeltaFrequency, time);
    }
}
