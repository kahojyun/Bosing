namespace Qynit.PulseGen;
internal sealed class ShiftFrequencyElement : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= new HashSet<int> { ChannelId };

    public int ChannelId { get; }
    public double DeltaFrequency { get; }

    public ShiftFrequencyElement(int channelId, double deltaFrequency)
    {
        ChannelId = channelId;
        DeltaFrequency = deltaFrequency;
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        return 0;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        return 0;
    }
}
