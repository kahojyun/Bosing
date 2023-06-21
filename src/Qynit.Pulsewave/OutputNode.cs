using CommunityToolkit.Diagnostics;

namespace Qynit.Pulsewave;
public class OutputNode : IFilterNode
{
    public OutputNode(int length, double sampleRate, double tStart)
    {
        SampleRate = sampleRate;
        TStart = tStart;
        Length = length;
    }

    public double SampleRate { get; }
    public double TStart { get; }
    public double TEnd => TStart + Length / SampleRate;
    public int Length { get; }
    public string? Name { get; set; }
    public IList<IFilterNode> Outputs { get; } = Array.Empty<IFilterNode>();
    public IList<IFilterNode> Inputs { get; } = new List<IFilterNode>();

    private Waveform? _waveform;

    public void Initialize()
    {
        _waveform = new Waveform(Length, SampleRate, TStart);
    }

    public void Complete()
    {
    }

    public void AddPulse(IPulseShape shape, double tStart, double width, double plateau, double amplitude, double frequency, double phase, double referenceTime)
    {
        Guard.IsNotNull(_waveform);
        using var envelope = Waveform.CreateFromRange(SampleRate, tStart, tStart + width + plateau);
        WaveformUtils.SampleWaveform(envelope, shape, tStart, width, plateau);
        WaveformUtils.AddPulseToWaveform(_waveform, envelope, amplitude, frequency, phase, referenceTime, 0);
    }

    public void AddWaveform(Waveform waveform, double tShift, double amplitude, double frequency, double phase, double referenceTime)
    {
        Guard.IsNotNull(_waveform);
        WaveformUtils.AddPulseToWaveform(_waveform, waveform, amplitude, frequency, phase, referenceTime, tShift);
    }

    public Waveform TakeWaveform()
    {
        Guard.IsNotNull(_waveform);
        var waveform = _waveform;
        _waveform = null;
        return waveform;
    }
}
