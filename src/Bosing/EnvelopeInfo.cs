using CommunityToolkit.Diagnostics;

namespace Bosing;

public readonly record struct EnvelopeInfo
{
    private readonly double _indexOffset;
    public double IndexOffset
    {
        get => _indexOffset;
        init
        {
            if (value < 0 || value >= 1)
            {
                ThrowHelper.ThrowArgumentOutOfRangeException(nameof(IndexOffset), value, $"Index offset of `EnvelopeInfo` should be in range [0, 1).");
            }
            _indexOffset = value;
        }
    }
    public double SampleRate { get; init; }

    public EnvelopeInfo(double indexOffset, double sampleRate)
    {
        IndexOffset = indexOffset;
        SampleRate = sampleRate;
    }
}
