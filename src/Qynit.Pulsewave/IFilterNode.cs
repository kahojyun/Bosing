using System.Numerics;

namespace Qynit.Pulsewave;
public interface IFilterNode<T>
    where T : unmanaged, IFloatingPointIeee754<T>
{
    void Initialize();
    void Complete();
    void AddPulse(IPulseShape shape, double tStart, double width, double plateau, double amplitude, double frequency, double phase, double referenceTime);
    void AddWaveform(ComplexArrayReadOnlySpan<T> waveform, WaveformInfo waveformInfo, double amplitude, double frequency, double phase, double referenceTime);
    double SampleRate { get; }
    double TStart { get; }
    double TEnd { get; }
    string? Name { get; set; }
    IList<IFilterNode<T>> Outputs { get; }
    IList<IFilterNode<T>> Inputs { get; }
}
