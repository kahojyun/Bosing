using System.Diagnostics;
using System.Numerics;

using BitFaster.Caching.Lru;

namespace Bosing;
public static class WaveformSampler<T>
    where T : unmanaged, IFloatingPointIeee754<T>
{
    private const int PlateauThreshold = 128;

    private static readonly FastConcurrentLru<EnvelopeCacheKey, EnvelopeSample<T>> MemoryCache = new(666);

    public static EnvelopeSample<T>? GetEnvelopeSample(EnvelopeInfo envelopeInfo, Envelope envelope)
    {
        var sampleRate = envelopeInfo.SampleRate;
        var shape = envelope.Shape;
        if (shape is null)
        {
            Debug.Assert(envelope.Width == 0);
            var rectLength = TimeAxisUtils.NextIndex(envelope.Plateau, sampleRate);
            return EnvelopeSample<T>.Rectangle(rectLength);
        }

        var dt = 1 / sampleRate;
        var tOffset = envelopeInfo.IndexOffset * dt;
        var width = envelope.Width;
        var plateau = envelope.Plateau;
        var t3 = width + plateau - tOffset;
        var length = TimeAxisUtils.NextIndex(t3, sampleRate);
        if (length == 0)
        {
            return null;
        }

        var key = new EnvelopeCacheKey(envelopeInfo, envelope);
        return MemoryCache.GetOrAdd(key, CreateEnvelopeSample);
    }

    private static EnvelopeSample<T> CreateEnvelopeSample(EnvelopeCacheKey key)
    {
        var envelopeInfo = key.EnvelopeInfo;
        var envelope = key.Envelope;
        var sampleRate = envelopeInfo.SampleRate;
        var shape = envelope.Shape;
        Debug.Assert(shape is not null);

        var dt = 1 / sampleRate;
        var tOffset = envelopeInfo.IndexOffset * dt;
        var width = envelope.Width;
        var plateau = envelope.Plateau;
        var t1 = width / 2 - tOffset;
        var t2 = width / 2 + plateau - tOffset;
        var t3 = width + plateau - tOffset;
        var length = TimeAxisUtils.NextIndex(t3, sampleRate);
        var plateauStartIndex = TimeAxisUtils.NextIndex(t1, sampleRate);
        var plateauEndIndex = TimeAxisUtils.NextIndex(t2, sampleRate);
        var plateauLength = plateauEndIndex - plateauStartIndex;
        var xStep = T.CreateChecked(dt / width);
        EnvelopeSample<T> envelopeSample;
        if (plateauLength < PlateauThreshold)
        {
            var array = new PooledComplexArray<T>(length, false);
            var x0 = T.CreateChecked(-t1 / width);
            if (plateau == 0)
            {
                shape.SampleIQ(array, x0, xStep);
            }
            else
            {
                shape.SampleIQ(array[..plateauStartIndex], x0, xStep);
                array[plateauStartIndex..plateauEndIndex].Fill(T.One, T.Zero);
                var x2 = T.CreateChecked((plateauEndIndex * dt - t2) / width);
                shape.SampleIQ(array[plateauEndIndex..], x2, xStep);
            }
            envelopeSample = EnvelopeSample<T>.Continuous(array);
        }
        else
        {
            var leftLength = plateauStartIndex;
            var rightLength = length - plateauEndIndex;
            var leftArray = new PooledComplexArray<T>(leftLength, false);
            var rightArray = new PooledComplexArray<T>(rightLength, false);
            var x0 = T.CreateChecked(-t1 / width);
            shape.SampleIQ(leftArray, x0, xStep);
            var x2 = T.CreateChecked((plateauEndIndex * dt - t2) / width);
            shape.SampleIQ(rightArray, x2, xStep);
            envelopeSample = EnvelopeSample<T>.WithPlateau(leftArray, rightArray, plateauLength);
        }
        return envelopeSample;
    }
}
