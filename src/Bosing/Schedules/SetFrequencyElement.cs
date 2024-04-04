namespace Bosing.Schedules;
public sealed class SetFrequencyElement(int channelId, double frequency) : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= [ChannelId];

    public int ChannelId { get; } = channelId;
    public double Frequency { get; } = frequency;

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
        phaseTrackingTransform.SetFrequency(ChannelId, Frequency, time);
    }
}
