namespace Qynit.Pulsewave;
public interface IFilterNode
{
    void Initialize();
    void Complete();
    void AddPulse(IPulseShape shape, double tStart, double width, double plateau, double amplitude, double frequency, double phase, double referenceTime);
    void AddWaveform(Waveform waveform, double tShift, double amplitude, double frequency, double phase, double referenceTime);
    double SampleRate { get; }
    double TStart { get; }
    double TEnd { get; }
    string? Name { get; set; }
    IList<IFilterNode> Outputs { get; }
    IList<IFilterNode> Inputs { get; }
}
