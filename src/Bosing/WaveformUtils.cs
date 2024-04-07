using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

using CommunityToolkit.Diagnostics;

namespace Bosing;
public static class WaveformUtils
{
    public static PooledComplexArray<T> SampleWaveform<T>(PulseList pulseList, double sampleRate, double loFrequency, int length, int alignLevel) where T : unmanaged, IFloatingPointIeee754<T>
    {
        var filterGroup = pulseList.Items.GroupBy(x => x.Key.Filter);
        PooledComplexArray<T>? waveform = null;
        foreach (var filter in filterGroup)
        {
            var tempWaveform = new PooledComplexArray<T>(length, true);
            foreach (var (binKey, bin) in filter)
            {
                foreach (var pulse in bin)
                {
                    var delay = pulseList.TimeOffset + binKey.Delay;
                    var time = pulse.Time;
                    var tStart = time + delay;
                    var iFracStart = TimeAxisUtils.NextFracIndex(tStart, sampleRate, alignLevel);
                    var iStart = (int)Math.Ceiling(iFracStart);
                    var envelopeInfo = new EnvelopeInfo(iStart - iFracStart, sampleRate);
                    var envelopeSample = WaveformSampler<T>.GetEnvelopeSample(envelopeInfo, binKey.Envelope);
                    if (envelopeSample is null)
                    {
                        continue;
                    }

                    var globalFrequency = binKey.GlobalFrequency - loFrequency;
                    var localFrequency = binKey.LocalFrequency;
                    var totalFrequency = globalFrequency + localFrequency;
                    var dt = 1 / sampleRate;
                    var phaseShift = Math.Tau * globalFrequency * (iStart * dt - delay);
                    var amplitude = pulse.Amplitude * pulseList.AmplitudeMultiplier * Complex.FromPolarCoordinates(1, phaseShift);
                    var complexAmplitude = amplitude.Amplitude;
                    var dragAmplitude = amplitude.DragAmplitude * sampleRate;
                    var dPhase = T.CreateChecked(Math.Tau * totalFrequency * dt);
                    MixAddEnvelope(tempWaveform[iStart..], envelopeSample, (IqPair<T>)complexAmplitude, (IqPair<T>)dragAmplitude, dPhase);
                }
            }
            var finalFilter = SignalFilter<double>.Concat(filter.Key, pulseList.Filter);
            finalFilter.Filter(tempWaveform.DataI);
            finalFilter.Filter(tempWaveform.DataQ);
            if (waveform is null)
            {
                waveform = tempWaveform;
            }
            else
            {
                Add<T>(waveform, tempWaveform);
                tempWaveform.Dispose();
            }
        }
        return waveform ?? new PooledComplexArray<T>(length, true);
    }

    public static void ConvertDoubleToFloat(ComplexSpan<float> target, ComplexReadOnlySpan<double> source)
    {
        if (target.Length != source.Length)
        {
            ThrowHelper.ThrowArgumentException("target.Length != source.Length");
        }
        ref var targetI = ref MemoryMarshal.GetReference(target.DataI);
        ref var targetQ = ref MemoryMarshal.GetReference(target.DataQ);
        ref var sourceI = ref MemoryMarshal.GetReference(source.DataI);
        ref var sourceQ = ref MemoryMarshal.GetReference(source.DataQ);
        var length = target.Length;
        for (var i = 0; i < length; i++)
        {
            Unsafe.Add(ref targetI, i) = (float)Unsafe.Add(ref sourceI, i);
            Unsafe.Add(ref targetQ, i) = (float)Unsafe.Add(ref sourceQ, i);
        }
    }

    public static void IqTransform<T>(ComplexSpan<T> target, T a, T b, T c, T d, T iOffset, T qOffset) where T : unmanaged, INumber<T>
    {
        var i = 0;
        ref var targetI = ref MemoryMarshal.GetReference(target.DataI);
        ref var targetQ = ref MemoryMarshal.GetReference(target.DataQ);
        var vSize = Vector<T>.Count;
        if (Vector.IsHardwareAccelerated)
        {
            var vA = new Vector<T>(a);
            var vB = new Vector<T>(b);
            var vC = new Vector<T>(c);
            var vD = new Vector<T>(d);
            var vIOffset = new Vector<T>(iOffset);
            var vQOffset = new Vector<T>(qOffset);

            for (; i < target.Length - vSize + 1; i += vSize)
            {
                var iVector = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i));
                var qVector = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i));
                var iResult = vA * iVector + vB * qVector + vIOffset;
                var qResult = vC * iVector + vD * qVector + vQOffset;
                Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i)) = iResult;
                Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i)) = qResult;
            }
        }
        for (; i < target.Length; i++)
        {
            var iResult = a * Unsafe.Add(ref targetI, i) + b * Unsafe.Add(ref targetQ, i) + iOffset;
            var qResult = c * Unsafe.Add(ref targetI, i) + d * Unsafe.Add(ref targetQ, i) + qOffset;
            Unsafe.Add(ref targetI, i) = iResult;
            Unsafe.Add(ref targetQ, i) = qResult;
        }
    }

    private static void MixAddEnvelope<T>(ComplexSpan<T> target, EnvelopeSample<T> envelopeSample, IqPair<T> complexAmplitude, IqPair<T> dragAmplitude, T dPhase) where T : unmanaged, IFloatingPointIeee754<T>
    {
        var currentIndex = 0;

        var leftEdge = envelopeSample.LeftEdge;
        MixAdd(target[currentIndex..], leftEdge, complexAmplitude, dragAmplitude, dPhase);
        ShiftIndex(leftEdge.Length);
        MixAddPlateau(target.Slice(currentIndex, envelopeSample.Plateau), complexAmplitude, dPhase);
        ShiftIndex(envelopeSample.Plateau);
        var rightEdge = envelopeSample.RightEdge;
        MixAdd(target[currentIndex..], rightEdge, complexAmplitude, dragAmplitude, dPhase);

        void ShiftIndex(int count)
        {
            currentIndex += count;
            var phaseInc = IqPair<T>.FromPolarCoordinates(T.One, dPhase * T.CreateChecked(count));
            complexAmplitude *= phaseInc;
            dragAmplitude *= phaseInc;
        }
    }

    public static void MixAddPlateau<T>(ComplexSpan<T> target, IqPair<T> amplitude, T dPhase)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        if (target.Length == 0)
        {
            return;
        }
        if (dPhase == T.Zero)
        {
            MixAddPlateauCore(target, amplitude);
        }
        else
        {
            MixAddPlateauFrequencyCore(target, amplitude, dPhase);
        }
    }

    public static void MixAdd<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude, IqPair<T> dragAmplitude, T dPhase)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        if (source.Length == 0)
        {
            return;
        }
        switch ((dPhase == T.Zero, dragAmplitude == IqPair<T>.Zero))
        {
            case (true, true):
                MixAddCore(target, source, amplitude);
                break;
            case (true, false):
                MixAddWithDragCore(target, source, amplitude, dragAmplitude);
                break;
            case (false, true):
                MixAddFrequencyCore(target, source, amplitude, dPhase);
                break;
            case (false, false):
                MixAddFrequencyWithDragCore(target, source, amplitude, dragAmplitude, dPhase);
                break;
        }
    }

    internal static void MixAddPlateauCore<T>(ComplexSpan<T> target, IqPair<T> amplitude)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = target.Length;

        var i = 0;
        ref var targetI = ref MemoryMarshal.GetReference(target.DataI);
        ref var targetQ = ref MemoryMarshal.GetReference(target.DataQ);
        var vSize = Vector<T>.Count;

        if (Vector.IsHardwareAccelerated && length >= 2 * vSize)
        {
            var amplitudeVectorI = new Vector<T>(amplitude.I);
            var amplitudeVectorQ = new Vector<T>(amplitude.Q);

            for (; i < length - vSize + 1; i += vSize)
            {
                ref var targetVectorI = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i));
                ref var targetVectorQ = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i));
                targetVectorI += amplitudeVectorI;
                targetVectorQ += amplitudeVectorQ;
            }
        }

        for (; i < length; i++)
        {
            Unsafe.Add(ref targetI, i) += amplitude.I;
            Unsafe.Add(ref targetQ, i) += amplitude.Q;
        }
    }

    internal static void MixAddPlateauFrequencyCore<T>(ComplexSpan<T> target, IqPair<T> amplitude, T dPhase)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = target.Length;
        var carrier = amplitude;
        var phaser = IqPair<T>.FromPolarCoordinates(T.One, dPhase);
        var i = 0;
        ref var targetI = ref MemoryMarshal.GetReference(target.DataI);
        ref var targetQ = ref MemoryMarshal.GetReference(target.DataQ);
        var vSize = Vector<T>.Count;

        if (Vector.IsHardwareAccelerated && length >= 2 * vSize)
        {
            Span<T> phaserI = stackalloc T[vSize];
            Span<T> phaserQ = stackalloc T[vSize];
            var phaserBulk = IqPair<T>.One;
            for (var j = 0; j < vSize; j++)
            {
                phaserI[j] = phaserBulk.I;
                phaserQ[j] = phaserBulk.Q;
                phaserBulk *= phaser;
            }
            var phaserVectorI = new Vector<T>(phaserI);
            var phaserVectorQ = new Vector<T>(phaserQ);

            var carrierBroadcastI = new Vector<T>(carrier.I);
            var carrierBroadcastQ = new Vector<T>(carrier.Q);
            var carrierVectorI = phaserVectorI * carrierBroadcastI - phaserVectorQ * carrierBroadcastQ;
            var carrierVectorQ = phaserVectorI * carrierBroadcastQ + phaserVectorQ * carrierBroadcastI;

            var phaserBulkBroadcastI = new Vector<T>(phaserBulk.I);
            var phaserBulkBroadcastQ = new Vector<T>(phaserBulk.Q);
            for (; i < length - vSize + 1; i += vSize)
            {
                ref var targetVectorI = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i));
                ref var targetVectorQ = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i));
                targetVectorI += carrierVectorI;
                targetVectorQ += carrierVectorQ;

                var newCarrierVectorI = carrierVectorI * phaserBulkBroadcastI - carrierVectorQ * phaserBulkBroadcastQ;
                var newCarrierVectorQ = carrierVectorI * phaserBulkBroadcastQ + carrierVectorQ * phaserBulkBroadcastI;
                carrierVectorI = newCarrierVectorI;
                carrierVectorQ = newCarrierVectorQ;
            }
            carrier = new IqPair<T>(carrierVectorI[0], carrierVectorQ[0]);
        }

        for (; i < length; i++)
        {
            Unsafe.Add(ref targetI, i) += carrier.I;
            Unsafe.Add(ref targetQ, i) += carrier.Q;
            carrier *= phaser;
        }
    }

    internal static void Add<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source)
        where T : unmanaged, INumber<T>
    {
        var length = source.Length;
        if (length == 0)
        {
            return;
        }
        LengthCheck(target, source);

        var i = 0;
        ref var targetI = ref MemoryMarshal.GetReference(target.DataI);
        ref var targetQ = ref MemoryMarshal.GetReference(target.DataQ);
        ref var sourceI = ref MemoryMarshal.GetReference(source.DataI);
        ref var sourceQ = ref MemoryMarshal.GetReference(source.DataQ);
        var vSize = Vector<T>.Count;

        if (Vector.IsHardwareAccelerated && length >= 2 * vSize)
        {
            for (; i < length - vSize + 1; i += vSize)
            {
                var sourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i));
                var sourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i));
                ref var targetVectorI = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i));
                ref var targetVectorQ = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i));
                targetVectorI += sourceVectorI;
                targetVectorQ += sourceVectorQ;
            }
        }

        for (; i < length; i++)
        {
            Unsafe.Add(ref targetI, i) += Unsafe.Add(ref sourceI, i);
            Unsafe.Add(ref targetQ, i) += Unsafe.Add(ref sourceQ, i);
        }
    }

    internal static void MixAddCore<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        if (length == 0)
        {
            return;
        }
        LengthCheck(target, source);

        var i = 0;
        ref var targetI = ref MemoryMarshal.GetReference(target.DataI);
        ref var targetQ = ref MemoryMarshal.GetReference(target.DataQ);
        ref var sourceI = ref MemoryMarshal.GetReference(source.DataI);
        ref var sourceQ = ref MemoryMarshal.GetReference(source.DataQ);
        var vSize = Vector<T>.Count;

        if (Vector.IsHardwareAccelerated && length >= 2 * vSize)
        {
            var amplitudeVectorI = new Vector<T>(amplitude.I);
            var amplitudeVectorQ = new Vector<T>(amplitude.Q);

            for (; i < length - vSize + 1; i += vSize)
            {
                var sourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i));
                var sourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i));
                var tempI = sourceVectorI * amplitudeVectorI - sourceVectorQ * amplitudeVectorQ;
                var tempQ = sourceVectorI * amplitudeVectorQ + sourceVectorQ * amplitudeVectorI;
                ref var targetVectorI = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i));
                ref var targetVectorQ = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i));
                targetVectorI += tempI;
                targetVectorQ += tempQ;
            }
        }

        for (; i < length; i++)
        {
            var sourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, i), Unsafe.Add(ref sourceQ, i));
            var targetIq = sourceIq * amplitude;
            Unsafe.Add(ref targetI, i) += targetIq.I;
            Unsafe.Add(ref targetQ, i) += targetIq.Q;
        }
    }

    internal static void MixAddFrequencyCore<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude, T dPhase)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        if (length == 0)
        {
            return;
        }
        LengthCheck(target, source);

        var carrier = amplitude;
        var phaser = IqPair<T>.FromPolarCoordinates(T.One, dPhase);
        var i = 0;
        ref var targetI = ref MemoryMarshal.GetReference(target.DataI);
        ref var targetQ = ref MemoryMarshal.GetReference(target.DataQ);
        ref var sourceI = ref MemoryMarshal.GetReference(source.DataI);
        ref var sourceQ = ref MemoryMarshal.GetReference(source.DataQ);
        var vSize = Vector<T>.Count;

        if (Vector.IsHardwareAccelerated && length >= 2 * vSize)
        {
            Span<T> phaserI = stackalloc T[vSize];
            Span<T> phaserQ = stackalloc T[vSize];
            var phaserBulk = IqPair<T>.One;
            for (var j = 0; j < vSize; j++)
            {
                phaserI[j] = phaserBulk.I;
                phaserQ[j] = phaserBulk.Q;
                phaserBulk *= phaser;
            }
            var phaserVectorI = new Vector<T>(phaserI);
            var phaserVectorQ = new Vector<T>(phaserQ);

            var carrierBroadcastI = new Vector<T>(carrier.I);
            var carrierBroadcastQ = new Vector<T>(carrier.Q);
            var carrierVectorI = phaserVectorI * carrierBroadcastI - phaserVectorQ * carrierBroadcastQ;
            var carrierVectorQ = phaserVectorI * carrierBroadcastQ + phaserVectorQ * carrierBroadcastI;

            var phaserBulkBroadcastI = new Vector<T>(phaserBulk.I);
            var phaserBulkBroadcastQ = new Vector<T>(phaserBulk.Q);
            for (; i < length - vSize + 1; i += vSize)
            {
                var sourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i));
                var sourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i));
                var tempI = sourceVectorI * carrierVectorI - sourceVectorQ * carrierVectorQ;
                var tempQ = sourceVectorI * carrierVectorQ + sourceVectorQ * carrierVectorI;
                ref var targetVectorI = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i));
                ref var targetVectorQ = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i));
                targetVectorI += tempI;
                targetVectorQ += tempQ;
                var newCarrierVectorI = carrierVectorI * phaserBulkBroadcastI - carrierVectorQ * phaserBulkBroadcastQ;
                var newCarrierVectorQ = carrierVectorI * phaserBulkBroadcastQ + carrierVectorQ * phaserBulkBroadcastI;
                carrierVectorI = newCarrierVectorI;
                carrierVectorQ = newCarrierVectorQ;
            }
            carrier = new IqPair<T>(carrierVectorI[0], carrierVectorQ[0]);
        }

        for (; i < length; i++)
        {
            var sourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, i), Unsafe.Add(ref sourceQ, i));
            var targetIq = sourceIq * carrier;
            Unsafe.Add(ref targetI, i) += targetIq.I;
            Unsafe.Add(ref targetQ, i) += targetIq.Q;
            carrier *= phaser;
        }
    }

    internal static void MixAddWithDragCore<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude, IqPair<T> dragAmplitude)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        if (length == 0)
        {
            return;
        }
        LengthCheck(target, source);

        ref var targetI = ref MemoryMarshal.GetReference(target.DataI);
        ref var targetQ = ref MemoryMarshal.GetReference(target.DataQ);
        ref var sourceI = ref MemoryMarshal.GetReference(source.DataI);
        ref var sourceQ = ref MemoryMarshal.GetReference(source.DataQ);

        // left boundary
        {
            var sourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, 0), Unsafe.Add(ref sourceQ, 0));
            var sourceWithAmplitudeIq = sourceIq * amplitude;
            if (length == 1)
            {
                Unsafe.Add(ref targetI, 0) += sourceWithAmplitudeIq.I;
                Unsafe.Add(ref targetQ, 0) += sourceWithAmplitudeIq.Q;
                return;
            }
            var nextSourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, 1), Unsafe.Add(ref sourceQ, 1));
            var diff = nextSourceIq - sourceIq;
            var dragWithAmplitudeIq = diff * dragAmplitude;
            var totalIq = sourceWithAmplitudeIq + dragWithAmplitudeIq;
            Unsafe.Add(ref targetI, 0) += totalIq.I;
            Unsafe.Add(ref targetQ, 0) += totalIq.Q;
        }
        var halfDragAmplitude = dragAmplitude * T.Exp2(-T.One);

        var i = 1;
        var vSize = Vector<T>.Count;
        if (Vector.IsHardwareAccelerated && length >= 2 * vSize + 2)
        {
            var amplitudeVectorI = new Vector<T>(amplitude.I);
            var amplitudeVectorQ = new Vector<T>(amplitude.Q);
            var halfDragAmplitudeVectorI = new Vector<T>(halfDragAmplitude.I);
            var halfDragAmplitudeVectorQ = new Vector<T>(halfDragAmplitude.Q);

            for (; i < length - vSize; i += vSize)
            {
                var sourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i));
                var sourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i));
                var sourceWithAmplitudeVectorI = sourceVectorI * amplitudeVectorI - sourceVectorQ * amplitudeVectorQ;
                var sourceWithAmplitudeVectorQ = sourceVectorI * amplitudeVectorQ + sourceVectorQ * amplitudeVectorI;

                var nextSourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i + 1));
                var nextSourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i + 1));
                var prevSourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i - 1));
                var prevSourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i - 1));
                var diffVectorI = nextSourceVectorI - prevSourceVectorI;
                var diffVectorQ = nextSourceVectorQ - prevSourceVectorQ;
                var dragWithAmplitudeVectorI = diffVectorI * halfDragAmplitudeVectorI - diffVectorQ * halfDragAmplitudeVectorQ;
                var dragWithAmplitudeVectorQ = diffVectorI * halfDragAmplitudeVectorQ + diffVectorQ * halfDragAmplitudeVectorI;

                var totalVectorI = sourceWithAmplitudeVectorI + dragWithAmplitudeVectorI;
                var totalVectorQ = sourceWithAmplitudeVectorQ + dragWithAmplitudeVectorQ;
                ref var targetVectorI = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i));
                ref var targetVectorQ = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i));
                targetVectorI += totalVectorI;
                targetVectorQ += totalVectorQ;
            }
        }

        for (; i < length - 1; i++)
        {
            var sourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, i), Unsafe.Add(ref sourceQ, i));
            var nextSourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, i + 1), Unsafe.Add(ref sourceQ, i + 1));
            var prevSourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, i - 1), Unsafe.Add(ref sourceQ, i - 1));
            var diff = nextSourceIq - prevSourceIq;
            var totalIq = sourceIq * amplitude + diff * halfDragAmplitude;
            Unsafe.Add(ref targetI, i) += totalIq.I;
            Unsafe.Add(ref targetQ, i) += totalIq.Q;
        }

        // right boundary
        {
            var sourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, length - 1), Unsafe.Add(ref sourceQ, length - 1));
            var prevSourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, length - 2), Unsafe.Add(ref sourceQ, length - 2));
            var diff = sourceIq - prevSourceIq;
            var totalIq = sourceIq * amplitude + diff * dragAmplitude;
            Unsafe.Add(ref targetI, length - 1) += totalIq.I;
            Unsafe.Add(ref targetQ, length - 1) += totalIq.Q;
        }
    }

    internal static void MixAddFrequencyWithDragCore<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude, IqPair<T> dragAmplitude, T dPhase)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        if (length == 0)
        {
            return;
        }
        LengthCheck(target, source);

        ref var targetI = ref MemoryMarshal.GetReference(target.DataI);
        ref var targetQ = ref MemoryMarshal.GetReference(target.DataQ);
        ref var sourceI = ref MemoryMarshal.GetReference(source.DataI);
        ref var sourceQ = ref MemoryMarshal.GetReference(source.DataQ);
        var carrier = amplitude;
        var dragCarrier = dragAmplitude;

        // left boundary
        {
            var sourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, 0), Unsafe.Add(ref sourceQ, 0));
            var sourceWithAmplitudeIq = sourceIq * carrier;
            if (length == 1)
            {
                Unsafe.Add(ref targetI, 0) += sourceWithAmplitudeIq.I;
                Unsafe.Add(ref targetQ, 0) += sourceWithAmplitudeIq.Q;
                return;
            }
            var nextSourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, 1), Unsafe.Add(ref sourceQ, 1));
            var diff = nextSourceIq - sourceIq;
            var dragWithAmplitudeIq = diff * dragCarrier;
            var totalIq = sourceWithAmplitudeIq + dragWithAmplitudeIq;
            Unsafe.Add(ref targetI, 0) += totalIq.I;
            Unsafe.Add(ref targetQ, 0) += totalIq.Q;
        }
        var phaser = IqPair<T>.FromPolarCoordinates(T.One, dPhase);
        carrier *= phaser;
        dragCarrier *= phaser * T.Exp2(-T.One);

        var i = 1;
        var vSize = Vector<T>.Count;
        if (Vector.IsHardwareAccelerated && length >= 2 * vSize + 2)
        {
            Span<T> phaserI = stackalloc T[vSize];
            Span<T> phaserQ = stackalloc T[vSize];
            var phaserBulk = IqPair<T>.One;
            for (var j = 0; j < vSize; j++)
            {
                phaserI[j] = phaserBulk.I;
                phaserQ[j] = phaserBulk.Q;
                phaserBulk *= phaser;
            }

            var phaserVectorI = new Vector<T>(phaserI);
            var phaserVectorQ = new Vector<T>(phaserQ);

            var carrierBroadcastI = new Vector<T>(carrier.I);
            var carrierBroadcastQ = new Vector<T>(carrier.Q);
            var dragCarrierBroadcastI = new Vector<T>(dragCarrier.I);
            var dragCarrierBroadcastQ = new Vector<T>(dragCarrier.Q);
            var carrierVectorI = phaserVectorI * carrierBroadcastI - phaserVectorQ * carrierBroadcastQ;
            var carrierVectorQ = phaserVectorI * carrierBroadcastQ + phaserVectorQ * carrierBroadcastI;
            var dragCarrierVectorI = phaserVectorI * dragCarrierBroadcastI - phaserVectorQ * dragCarrierBroadcastQ;
            var dragCarrierVectorQ = phaserVectorI * dragCarrierBroadcastQ + phaserVectorQ * dragCarrierBroadcastI;

            var phaserBulkBroadcastI = new Vector<T>(phaserBulk.I);
            var phaserBulkBroadcastQ = new Vector<T>(phaserBulk.Q);
            for (; i < length - vSize; i += vSize)
            {
                var sourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i));
                var sourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i));
                var sourceWithAmplitudeVectorI = sourceVectorI * carrierVectorI - sourceVectorQ * carrierVectorQ;
                var sourceWithAmplitudeVectorQ = sourceVectorI * carrierVectorQ + sourceVectorQ * carrierVectorI;

                var nextSourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i + 1));
                var nextSourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i + 1));
                var prevSourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i - 1));
                var prevSourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i - 1));
                var diffVectorI = nextSourceVectorI - prevSourceVectorI;
                var diffVectorQ = nextSourceVectorQ - prevSourceVectorQ;
                var dragWithAmplitudeVectorI = diffVectorI * dragCarrierVectorI - diffVectorQ * dragCarrierVectorQ;
                var dragWithAmplitudeVectorQ = diffVectorI * dragCarrierVectorQ + diffVectorQ * dragCarrierVectorI;

                var totalVectorI = sourceWithAmplitudeVectorI + dragWithAmplitudeVectorI;
                var totalVectorQ = sourceWithAmplitudeVectorQ + dragWithAmplitudeVectorQ;
                ref var targetVectorI = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i));
                ref var targetVectorQ = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i));
                targetVectorI += totalVectorI;
                targetVectorQ += totalVectorQ;

                var newCarrierVectorI = carrierVectorI * phaserBulkBroadcastI - carrierVectorQ * phaserBulkBroadcastQ;
                var newCarrierVectorQ = carrierVectorI * phaserBulkBroadcastQ + carrierVectorQ * phaserBulkBroadcastI;
                var newDragCarrierVectorI = dragCarrierVectorI * phaserBulkBroadcastI - dragCarrierVectorQ * phaserBulkBroadcastQ;
                var newDragCarrierVectorQ = dragCarrierVectorI * phaserBulkBroadcastQ + dragCarrierVectorQ * phaserBulkBroadcastI;
                carrierVectorI = newCarrierVectorI;
                carrierVectorQ = newCarrierVectorQ;
                dragCarrierVectorI = newDragCarrierVectorI;
                dragCarrierVectorQ = newDragCarrierVectorQ;
            }
            carrier = new IqPair<T>(carrierVectorI[0], carrierVectorQ[0]);
            dragCarrier = new IqPair<T>(dragCarrierVectorI[0], dragCarrierVectorQ[0]);
        }

        for (; i < length - 1; i++)
        {
            var sourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, i), Unsafe.Add(ref sourceQ, i));
            var nextSourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, i + 1), Unsafe.Add(ref sourceQ, i + 1));
            var prevSourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, i - 1), Unsafe.Add(ref sourceQ, i - 1));
            var diff = nextSourceIq - prevSourceIq;
            var totalIq = sourceIq * carrier + diff * dragCarrier;
            Unsafe.Add(ref targetI, i) += totalIq.I;
            Unsafe.Add(ref targetQ, i) += totalIq.Q;
            carrier *= phaser;
            dragCarrier *= phaser;
        }

        // right boundary
        {
            dragCarrier *= T.Exp2(T.One);
            var sourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, length - 1), Unsafe.Add(ref sourceQ, length - 1));
            var prevSourceIq = new IqPair<T>(Unsafe.Add(ref sourceI, length - 2), Unsafe.Add(ref sourceQ, length - 2));
            var diff = sourceIq - prevSourceIq;
            var totalIq = sourceIq * carrier + diff * dragCarrier;
            Unsafe.Add(ref targetI, length - 1) += totalIq.I;
            Unsafe.Add(ref targetQ, length - 1) += totalIq.Q;
        }
    }

    private static void LengthCheck<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source) where T : unmanaged
    {
        Debug.Assert(target.Length >= source.Length);
        if (target.Length < source.Length)
        {
            ThrowHelper.ThrowArgumentException("Target length must be greater than or equal to source length.");
        }
    }
}
