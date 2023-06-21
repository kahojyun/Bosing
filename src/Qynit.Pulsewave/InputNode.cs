namespace Qynit.Pulsewave;
public class InputNode : FilterNodeBase
{
    public override void AddPulse(IPulseShape shape, double tStart, double width, double plateau, double amplitude, double frequency, double phase, double referenceTime)
    {
        foreach (var output in Outputs)
        {
            output.AddPulse(shape, tStart, width, plateau, amplitude, frequency, phase, referenceTime);
        }
    }

    public override void AddWaveform(Waveform waveform, double tShift, double amplitude, double frequency, double phase, double referenceTime)
    {
        foreach (var output in Outputs)
        {
            output.AddWaveform(waveform, tShift, amplitude, frequency, phase, referenceTime);
        }
    }
}
