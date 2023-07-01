using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Qynit.Pulsewave;
public static class WaveformUtils
{
    public static PooledComplexArray<T> SampleEnvelope<T>(EnvelopeInfo envelopeInfo, IPulseShape shape, double width, double plateau)
       where T : unmanaged, IFloatingPointIeee754<T>
    {
        var sampleRate = envelopeInfo.SampleRate;
        var dt = 1 / sampleRate;
        var tOffset = envelopeInfo.IndexOffset * dt;
        var t1 = width / 2 - tOffset;
        var t2 = width / 2 + plateau - tOffset;
        var t3 = width + plateau - tOffset;
        var length = TimeAxisUtils.NextIndex(t3, sampleRate);
        var array = new PooledComplexArray<T>(length, false);

        var xStep = T.CreateChecked(dt / width);

        var x0 = T.CreateChecked(-t1 / width);
        var plateauStartIndex = TimeAxisUtils.NextIndex(t1, sampleRate);
        shape.SampleIQ(array[..plateauStartIndex], x0, xStep);

        int plateauEndIndex;
        if (plateau > 0)
        {
            plateauEndIndex = TimeAxisUtils.NextIndex(t2, sampleRate);
            array.DataI[plateauStartIndex..plateauEndIndex].Fill(T.One);
            array.DataQ[plateauStartIndex..plateauEndIndex].Clear();
        }
        else
        {
            plateauEndIndex = plateauStartIndex;
        }

        var x2 = T.CreateChecked((plateauEndIndex * dt - t2) / width);
        shape.SampleIQ(array[plateauEndIndex..], x2, xStep);

        return array;
    }

    internal static PooledComplexArray<T> SampleWaveform<T>(PulseList<T> pulseList, double sampleRate, int length, int alignLevel) where T : unmanaged, IFloatingPointIeee754<T>
    {
        var waveform = new PooledComplexArray<T>(length, true);
        foreach (var (binKey, bin) in pulseList.Items)
        {
            foreach (var pulse in bin)
            {
                var tStart = pulse.Time + pulseList.TimeOffset;
                var shape = binKey.Envelope.Shape;
                var width = binKey.Envelope.Width;
                var plateau = binKey.Envelope.Plateau;
                var frequency = binKey.Frequency;
                var iFracStart = TimeAxisUtils.NextFracIndex(tStart, sampleRate, alignLevel);
                var iStart = (int)Math.Ceiling(iFracStart);
                var envelopeInfo = new EnvelopeInfo(iStart - iFracStart, sampleRate);
                var dt = 1 / sampleRate;
                var amplitude = pulse.Amplitude * pulseList.AmplitudeMultiplier * IqPair<T>.FromPolarCoordinates(T.One, T.CreateChecked((iStart * dt - tStart) * frequency * Math.Tau));
                var complexAmplitude = amplitude.Amplitude;
                var dPhase = T.CreateChecked(Math.Tau * frequency * dt);
                var arrayIStart = iStart;
                var dragAmplitude = amplitude.DragAmplitude;
                if (shape is null)
                {
                    var plateauLength = (int)Math.Ceiling((width + plateau) * sampleRate);
                    if (frequency == 0)
                    {
                        MixAddPlateau(waveform.Slice(iStart, plateauLength), complexAmplitude);
                    }
                    else
                    {
                        MixAddPlateauFrequency(waveform.Slice(iStart, plateauLength), complexAmplitude, dPhase);
                    }
                }
                else
                {
                    using var envelope = SampleEnvelope<T>(envelopeInfo, shape, width, plateau);
                    if (dragAmplitude == IqPair<T>.Zero)
                    {
                        if (frequency == 0)
                        {
                            MixAdd(waveform[iStart..], envelope, complexAmplitude);
                        }
                        else
                        {
                            MixAddFrequency(waveform[iStart..], envelope, complexAmplitude, dPhase);
                        }
                    }
                    else
                    {
                        var drag = dragAmplitude * T.CreateChecked(sampleRate);
                        if (frequency == 0)
                        {
                            MixAddWithDrag(waveform[iStart..], envelope, complexAmplitude, drag);
                        }
                        else
                        {
                            MixAddFrequencyWithDrag(waveform[iStart..], envelope, complexAmplitude, drag, dPhase);
                        }
                    }
                }
            }
        }
        return waveform;
    }

    public static void MixAddPlateau<T>(ComplexSpan<T> target, IqPair<T> amplitude)
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

    public static void MixAddPlateauFrequency<T>(ComplexSpan<T> target, IqPair<T> amplitude, T dPhase)
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

    public static void MixAdd<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        if (length == 0)
        {
            return;
        }
        Debug.Assert(target.Length >= source.Length);

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

    public static void MixAddFrequency<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude, T dPhase)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        if (length == 0)
        {
            return;
        }
        Debug.Assert(target.Length >= source.Length);

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

    public static void MixAddWithDrag<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude, IqPair<T> dragAmplitude)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        if (length == 0)
        {
            return;
        }
        Debug.Assert(target.Length >= source.Length);
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

    public static void MixAddFrequencyWithDrag<T>(ComplexSpan<T> target, ComplexReadOnlySpan<T> source, IqPair<T> amplitude, IqPair<T> dragAmplitude, T dPhase)
        where T : unmanaged, IFloatingPointIeee754<T>
    {
        var length = source.Length;
        if (length == 0)
        {
            return;
        }
        Debug.Assert(target.Length >= source.Length);
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
}
