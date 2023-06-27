using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Qynit.Pulsewave;
public static class WaveformUtils
{
    public static PooledComplexArray<T> SampleWaveform<T>(EnvelopeInfo envelopeInfo, IPulseShape shape, double width, double plateau)
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

    public static void MixAddFrequency<T>(ComplexArraySpan<T> target, ComplexArrayReadOnlySpan<T> source, IqPair<T> amplitude, T dPhase)
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

            for (; i < length - vSize + 1; i += vSize)
            {
                var sourceVectorI = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceI, i));
                var sourceVectorQ = Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref sourceQ, i));
                var tempI = sourceVectorI * phaserVectorI - sourceVectorQ * phaserVectorQ;
                var tempQ = sourceVectorI * phaserVectorQ + sourceVectorQ * phaserVectorI;
                ref var targetVectorI = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetI, i));
                ref var targetVectorQ = ref Unsafe.As<T, Vector<T>>(ref Unsafe.Add(ref targetQ, i));
                targetVectorI += tempI * carrier.I - tempQ * carrier.Q;
                targetVectorQ += tempI * carrier.Q + tempQ * carrier.I;
                carrier *= phaserBulk;
            }
        }

        for (; i < length; i++)
        {
            var sourceScalarI = Unsafe.Add(ref sourceI, i);
            var sourceScalarQ = Unsafe.Add(ref sourceQ, i);
            ref var targetScalarI = ref Unsafe.Add(ref targetI, i);
            ref var targetScalarQ = ref Unsafe.Add(ref targetQ, i);
            targetScalarI += sourceScalarI * carrier.I - sourceScalarQ * carrier.Q;
            targetScalarQ += sourceScalarI * carrier.Q + sourceScalarQ * carrier.I;
            carrier *= phaser;
        }
    }
}
