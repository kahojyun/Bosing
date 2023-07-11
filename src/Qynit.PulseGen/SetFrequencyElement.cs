namespace Qynit.PulseGen;
internal sealed class SetFrequencyElement : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= new HashSet<int> { ChannelId };

    public int ChannelId { get; }
    public double Frequency { get; }

    public SetFrequencyElement(int channelId, double frequency)
    {
        ChannelId = channelId;
        Frequency = frequency;
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
