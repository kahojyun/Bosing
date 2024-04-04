using System.Diagnostics;

namespace Bosing.Schedules;
public sealed class PlayElement : ScheduleElement
{
    private HashSet<int>? _channels;
    public override IReadOnlySet<int> Channels => _channels ??= [ChannelId];
    public bool FlexiblePlateau { get; set; }
    public int ChannelId { get; }
    public IPulseShape? PulseShape { get; }
    public double Width { get; }
    public double Plateau { get; }
    public double Frequency { get; }
    public double Phase { get; }
    public double Amplitude { get; }
    public double DragCoefficient { get; }

    public PlayElement(int channelId, Envelope envelope, double frequency, double phase, double amplitude, double dragCoefficient)
    {
        Debug.Assert(envelope.Duration >= 0);
        ChannelId = channelId;
        PulseShape = envelope.Shape;
        Width = envelope.Width;
        Plateau = envelope.Plateau;
        Frequency = frequency;
        Phase = phase;
        Amplitude = amplitude;
        DragCoefficient = dragCoefficient;
    }

    protected override double ArrangeOverride(double time, double finalDuration)
    {
        return FlexiblePlateau ? finalDuration : Plateau + Width;
    }

    protected override double MeasureOverride(double maxDuration)
    {
        return FlexiblePlateau ? Width : Plateau + Width;
    }

    protected override void RenderOverride(double time, PhaseTrackingTransform phaseTrackingTransform)
    {
        Debug.Assert(ActualDuration is not null);
        var plateau = FlexiblePlateau ? ActualDuration.Value - Width : Plateau;
        var envelope = new Envelope(PulseShape, Width, plateau);
        phaseTrackingTransform.Play(ChannelId, envelope, Frequency, Phase, Amplitude, DragCoefficient, time);
    }
}
