namespace Bosing.Tests;

public class EnvelopeInfoTests
{
    [Fact]
    public void Ctor_Normal_Equal()
    {
        // Arrange
        var offset = 0.4;
        var sampleRate = 2e9;
        var envelopeInfo = new EnvelopeInfo(offset, sampleRate);

        // Assert
        Assert.Equal(offset, envelopeInfo.IndexOffset);
        Assert.Equal(sampleRate, envelopeInfo.SampleRate);
    }

    [Fact]
    public void Ctor_OffsetOutOfRange_Throw()
    {
        var sampleRate = 2e9;
        var offset = -0.4;
        Assert.Throws<ArgumentOutOfRangeException>(() => new EnvelopeInfo(offset, sampleRate));
        var offset2 = 1;
        Assert.Throws<ArgumentOutOfRangeException>(() => new EnvelopeInfo(offset2, sampleRate));
    }

    [Fact]
    public void Comparison_NotEqual()
    {
        var sampleRate = 2e9;
        var offset = 0.3;
        var offset2 = 0.4;
        var envelopeInfo = new EnvelopeInfo(offset, sampleRate);
        var envelopeInfo2 = new EnvelopeInfo(offset2, sampleRate);
        Assert.True(envelopeInfo != envelopeInfo2);
    }

    [Fact]
    public void Comparison_Equal()
    {
        var sampleRate = 2e9;
        var offset = 0.3;
        var offset2 = 0.3;
        var envelopeInfo = new EnvelopeInfo(offset, sampleRate);
        var envelopeInfo2 = new EnvelopeInfo(offset2, sampleRate);
        Assert.True(envelopeInfo == envelopeInfo2);
    }

    [Fact]
    public void Hash_SameItem()
    {
        var sampleRate = 2e9;
        var offset = 0.3;
        var offset2 = 0.3;
        var envelopeInfo = new EnvelopeInfo(offset, sampleRate);
        var envelopeInfo2 = new EnvelopeInfo(offset2, sampleRate);
        var hashSet = new HashSet<EnvelopeInfo>() { envelopeInfo, envelopeInfo2 };

        Assert.Single(hashSet);
    }

    [Fact]
    public void Hash_DifferentItem()
    {
        var sampleRate = 2e9;
        var offset = 0.3;
        var offset2 = 0.4;
        var envelopeInfo = new EnvelopeInfo(offset, sampleRate);
        var envelopeInfo2 = new EnvelopeInfo(offset2, sampleRate);
        var hashSet = new HashSet<EnvelopeInfo>() { envelopeInfo, envelopeInfo2 };

        Assert.Equal(2, hashSet.Count);
    }
}
