using System.Diagnostics;
using System.Numerics;

namespace Qynit.Pulsewave;
internal class SimpleWaveformOperator : IWaveformOperator
{
    private const double WaveformAlignErr = 1e-3;

    public void AddPulseToWaveform(Waveform target, Waveform pulse, double amplitude, double frequency, double phase, double referenceTime)
    {
        Debug.Assert(target.SampleRate == pulse.SampleRate);
        Debug.Assert(target.TStart <= pulse.TStart);
        Debug.Assert(target.TEnd >= pulse.TEnd);
        var startSample = (pulse.TStart - target.TStart) * target.SampleRate;
        var startIndex = (int)Math.Round(startSample);
        Debug.Assert(Math.Abs(startSample - startIndex) < WaveformAlignErr);
        var targetDataI = target.DataI;
        var targetDataQ = target.DataQ;
        var pulseDataI = pulse.DataI;
        var pulseDataQ = pulse.DataQ;
        var startPhase = phase + Math.Tau * frequency * (pulse.TStart - referenceTime);
        var deltaPhase = Math.Tau * frequency * pulse.Dt;
        var carrier = Complex.FromPolarCoordinates(amplitude, startPhase);
        var deltaCarrier = Complex.FromPolarCoordinates(1, deltaPhase);
        for (var i = 0; i < pulse.Length; i++)
        {
            var targetPoint = new Complex(targetDataI[startIndex + i], targetDataQ[startIndex + i]);
            var pulsePoint = new Complex(pulseDataI[i], pulseDataQ[i]);
            var resultPoint = targetPoint + carrier * pulsePoint;
            targetDataI[startIndex + i] = resultPoint.Real;
            targetDataQ[startIndex + i] = resultPoint.Imaginary;
            carrier *= deltaCarrier;
        }
    }

    public void SampleWaveform(Waveform target, IPulseShape shape, double tStart, double width, double plateau)
    {
        // inclusive
        var sampleStartIndex = (int)Math.Ceiling((tStart - target.TStart) * target.SampleRate);
        // exclusive
        var sampleEndIndex = (int)Math.Ceiling((tStart + width + plateau - target.TStart) * target.SampleRate);
        Debug.Assert(sampleStartIndex >= 0);
        Debug.Assert(sampleEndIndex < target.Length);
        // inclusive
        var plateauStartIndex = (int)Math.Ceiling((tStart + width / 2 - target.TStart) * target.SampleRate);
        // exclusive
        var plateauEndIndex = (int)Math.Ceiling((tStart + width / 2 + plateau - target.TStart) * target.SampleRate);
        // rising edge
        for (var i = sampleStartIndex; i < plateauStartIndex; i++)
        {
            var t = target.TimeAt(i);
            var x = (t - tStart) / width - 0.5;
            var y = shape.SampleAt(x);
            target.DataI[i] = y.Real;
            target.DataQ[i] = y.Imaginary;
        }
        // plateau
        for (var i = plateauStartIndex; i < plateauEndIndex; i++)
        {
            target.DataI[i] = 1;
            target.DataQ[i] = 0;
        }
        // falling edge
        for (var i = plateauEndIndex; i < sampleEndIndex; i++)
        {
            var t = target.TimeAt(i);
            var x = (t - tStart - width / 2 - plateau) / width;
            var y = shape.SampleAt(x);
            target.DataI[i] = y.Real;
            target.DataQ[i] = y.Imaginary;
        }
    }
}
