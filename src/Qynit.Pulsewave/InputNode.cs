using System.Numerics;

namespace Qynit.Pulsewave;
public class InputNode<T> : FilterNodeBase<T>
    where T : unmanaged, IFloatingPointIeee754<T>
{
    public override void AddPulse(IPulseShape shape, double tStart, double width, double plateau, double amplitude, double frequency, double phase, double referenceTime)
    {
        foreach (var output in Outputs)
        {
            output.AddPulse(shape, tStart, width, plateau, amplitude, frequency, phase, referenceTime);
        }
    }

    public override void AddWaveform(ComplexArrayReadOnlySpan<T> waveform, WaveformInfo waveformInfo, double amplitude, double frequency, double phase, double referenceTime)
    {
        foreach (var output in Outputs)
        {
            output.AddWaveform(waveform, waveformInfo, amplitude, frequency, phase, referenceTime);
        }
    }
}
