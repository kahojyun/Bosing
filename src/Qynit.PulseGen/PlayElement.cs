using System.Diagnostics;

namespace Qynit.PulseGen;
internal sealed class PlayElement : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= new HashSet<int> { ChannelId };
    public int ChannelId { get; }
    public Envelope Envelope { get; }
    public double Frequency { get; }
    public double Phase { get; }
    public double Amplitude { get; }
    public double DragCoefficient { get; }

    public PlayElement(int channelId, Envelope envelope, double frequency, double phase, double amplitude, double dragCoefficient)
    {
        Debug.Assert(envelope.Duration >= 0);
        ChannelId = channelId;
        Envelope = envelope;
        Frequency = frequency;
        Phase = phase;
        Amplitude = amplitude;
        DragCoefficient = dragCoefficient;
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        return Envelope.Duration;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        return Envelope.Duration;
    }
}
