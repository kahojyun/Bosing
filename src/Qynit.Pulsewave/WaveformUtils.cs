using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Qynit.Pulsewave;
public static class WaveformUtils
{
    private const double WaveformAlignErr = 1e-3;

    public static void SampleWaveform(Waveform target, IPulseShape shape, double tStart, double width, double plateau)
    {
        var t0 = tStart;
        var t1 = tStart + width / 2;
        var t2 = tStart + width / 2 + plateau;
        var t3 = tStart + width + plateau;
        var sampleStartIndex = (int)Math.Ceiling((t0 - target.TStart) * target.SampleRate);
        var plateauStartIndex = (int)Math.Ceiling((t1 - target.TStart) * target.SampleRate);
        var plateauEndIndex = (int)Math.Ceiling((t2 - target.TStart) * target.SampleRate);
        var sampleEndIndex = (int)Math.Ceiling((t3 - target.TStart) * target.SampleRate);
        Debug.Assert(sampleStartIndex >= 0);
        Debug.Assert(sampleEndIndex < target.Length);

        var dataI = target.DataI;
        var dataQ = target.DataQ;
        var xStep = target.Dt / width;

        var tStartRising = target.TimeAt(sampleStartIndex);
        var xStartRising = (tStartRising - t1) / width;
        var dataIRising = dataI[sampleStartIndex..plateauStartIndex];
        var dataQRising = dataQ[sampleStartIndex..plateauStartIndex];
        shape.SampleIQ(dataIRising, dataQRising, xStartRising, xStep);

        dataI[plateauStartIndex..plateauEndIndex].Fill(1);
        dataQ[plateauStartIndex..plateauEndIndex].Clear();

        var tStartFalling = target.TimeAt(plateauEndIndex);
        var xStartFalling = (tStartFalling - t2) / width;
        var dataIFalling = dataI[plateauEndIndex..sampleEndIndex];
        var dataQFalling = dataQ[plateauEndIndex..sampleEndIndex];
        shape.SampleIQ(dataIFalling, dataQFalling, xStartFalling, xStep);
    }

    public static void AddPulseToWaveform(Waveform target, Waveform pulse, double amplitude, double frequency, double phase, double referenceTime, double tShift)
    {
        Debug.Assert(target.SampleRate == pulse.SampleRate);
        tShift = MathUtils.MRound(tShift, pulse.Dt);
        var tStart = pulse.TStart + tShift;
        var tEnd = pulse.TEnd + tShift;
        Debug.Assert(target.TStart <= tStart);
        Debug.Assert(target.TEnd >= tEnd);
        var startSample = (tStart - target.TStart) * target.SampleRate;
        var startIndex = (int)Math.Round(startSample);
        Debug.Assert(Math.Abs(startSample - startIndex) < WaveformAlignErr);

        var targetDataI = target.DataI[startIndex..];
        var targetDataQ = target.DataQ[startIndex..];
        var pulseDataI = pulse.DataI;
        var pulseDataQ = pulse.DataQ;
        var startPhase = phase + Math.Tau * frequency * (tStart - referenceTime);
        var deltaPhase = Math.Tau * frequency * pulse.Dt;
        MixAndAddIQVector(targetDataI, targetDataQ, pulseDataI, pulseDataQ, amplitude, startPhase, deltaPhase);
    }

    internal static void MixAndAddIQVector(Span<double> targetI, Span<double> targetQ, ReadOnlySpan<double> sourceI, ReadOnlySpan<double> sourceQ, double amplitude, double phase, double dPhase)
    {
        var l = sourceI.Length;
        Debug.Assert(sourceQ.Length == l);
        Debug.Assert(targetI.Length >= l);
        Debug.Assert(targetQ.Length >= l);

        var c = Complex.FromPolarCoordinates(amplitude, phase);
        var w = Complex.FromPolarCoordinates(1, dPhase);
        var ii = 0;
        ref var ti = ref MemoryMarshal.GetReference(targetI);
        ref var tq = ref MemoryMarshal.GetReference(targetQ);
        ref var si = ref MemoryMarshal.GetReference(sourceI);
        ref var sq = ref MemoryMarshal.GetReference(sourceQ);

        if (Vector.IsHardwareAccelerated && l >= 2 * Vector<double>.Count)
        {
            Span<double> ci = stackalloc double[Vector<double>.Count];
            Span<double> cq = stackalloc double[Vector<double>.Count];
            var ww = Complex.One;
            for (int i = 0; i < Vector<double>.Count; i++)
            {
                ci[i] = ww.Real;
                cq[i] = ww.Imaginary;
                ww *= w;
            }
            var civ = new Vector<double>(ci);
            var cqv = new Vector<double>(cq);

            for (; ii < l - Vector<double>.Count + 1; ii += Vector<double>.Count)
            {
                ref var tiv = ref Unsafe.As<double, Vector<double>>(ref Unsafe.Add(ref ti, ii));
                ref var tqv = ref Unsafe.As<double, Vector<double>>(ref Unsafe.Add(ref tq, ii));
                ref var siv = ref Unsafe.As<double, Vector<double>>(ref Unsafe.Add(ref si, ii));
                ref var sqv = ref Unsafe.As<double, Vector<double>>(ref Unsafe.Add(ref sq, ii));
                var mi = siv * civ - sqv * cqv;
                var mq = siv * cqv + sqv * civ;
                (mi, mq) = (mi * c.Real - mq * c.Imaginary, mi * c.Imaginary + mq * c.Real);
                tiv += mi;
                tqv += mq;
                c *= ww;
            }
        }

        for (; ii < l; ii++)
        {
            var s = new Complex(Unsafe.Add(ref si, ii), Unsafe.Add(ref sq, ii));
            var t = new Complex(Unsafe.Add(ref ti, ii), Unsafe.Add(ref tq, ii));
            t += s * c;
            Unsafe.Add(ref ti, ii) = t.Real;
            Unsafe.Add(ref tq, ii) = t.Imaginary;
            c *= w;
        }
    }
}
