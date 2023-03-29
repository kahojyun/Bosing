namespace Qynit.Pulsewave;

/// <summary>
/// Peforms operations on waveform.
/// </summary>
internal interface IWaveformOperator
{
    /// <summary>
    /// Sample waveform with a pulse shape.
    /// </summary>
    /// <param name="target">Where to store the waveform</param>
    /// <param name="shape">Pulse shape</param>
    /// <param name="tStart">Start time of the pulse</param>
    /// <param name="width">Width of the pulse</param>
    /// <param name="plateau">Plateau of the pulse</param>
    void SampleWaveform(Waveform target, IPulseShape shape, double tStart, double width, double plateau);
    /// <summary>
    /// Add a pulse to a waveform and perform phase modulation.
    /// </summary>
    /// <param name="target">The waveform to add to</param>
    /// <param name="pulse">The waveform of the pulse</param>
    /// <param name="amplitude">Amplitude of the pulse</param>
    /// <param name="frequency">Frequency of the modulation (in Hz)</param>
    /// <param name="phase">Global phase of the pulse (in rad)</param>
    /// <param name="referenceTime">Reference time for phase modulation</param>
    void AddPulseToWaveform(Waveform target, Waveform pulse, double amplitude, double frequency, double phase, double referenceTime);
}
