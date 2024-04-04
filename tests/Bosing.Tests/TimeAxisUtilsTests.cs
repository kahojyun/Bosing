namespace Bosing.Tests;

public class TimeAxisUtilsTests
{
    private const double SampleRate = 1e9;

    [Theory]
    [InlineData(0e-9, 0)]
    [InlineData(0.5e-9, 1)]
    [InlineData(1e-9, 1)]
    [InlineData(10.5e-9, 11)]
    [InlineData(-10.5e-9, -10)]
    public void NextIndex_Normal_EqualExpected(double t, int ans)
    {
        // Arrange
        var sampleRate = SampleRate;

        // Act
        var result = TimeAxisUtils.NextIndex(
            t,
            sampleRate);

        // Assert
        Assert.Equal(ans, result);
    }

    [Theory]
    [InlineData(0, -1, 0)]
    [InlineData(0.25e-9, -1, 0.5)]
    [InlineData(0.5e-9, -1, 0.5)]
    [InlineData(0.8e-9, -1, 1)]
    [InlineData(10.8e-9, -1, 11)]
    [InlineData(-10.8e-9, -1, -10.5)]
    [InlineData(0, -2, 0)]
    [InlineData(0.25e-9, -2, 0.25)]
    [InlineData(0.5e-9, -2, 0.5)]
    [InlineData(0.8e-9, -2, 1)]
    [InlineData(10.8e-9, -2, 11)]
    [InlineData(-10.8e-9, -2, -10.75)]
    public void NextFracIndex_Normal_EqualExpected(double t, int alignLevel, double ans)
    {
        // Arrange
        var sampleRate = SampleRate;

        // Act
        var result = TimeAxisUtils.NextFracIndex(
            t,
            sampleRate,
            alignLevel);

        // Assert
        Assert.Equal(ans, result);
    }

    [Theory]
    [InlineData(0e-9, 0)]
    [InlineData(0.5e-9, 0)]
    [InlineData(1e-9, 1)]
    [InlineData(10.5e-9, 10)]
    [InlineData(-10.5e-9, -11)]
    public void PrevIndex_Normal_EqualExpected(double t, int ans)
    {
        // Arrange
        var sampleRate = SampleRate;

        // Act
        var result = TimeAxisUtils.PrevIndex(
            t,
            sampleRate);

        // Assert
        Assert.Equal(ans, result);
    }

    [Theory]
    [InlineData(0, -1, 0)]
    [InlineData(0.25e-9, -1, 0)]
    [InlineData(0.5e-9, -1, 0.5)]
    [InlineData(0.8e-9, -1, 0.5)]
    [InlineData(10.8e-9, -1, 10.5)]
    [InlineData(-10.8e-9, -1, -11)]
    [InlineData(0, -2, 0)]
    [InlineData(0.25e-9, -2, 0.25)]
    [InlineData(0.5e-9, -2, 0.5)]
    [InlineData(0.8e-9, -2, 0.75)]
    [InlineData(10.8e-9, -2, 10.75)]
    [InlineData(-10.8e-9, -2, -11)]
    public void PrevFracIndex_Normal_EqualExpected(double t, int alignLevel, double ans)
    {
        // Arrange
        var sampleRate = SampleRate;

        // Act
        var result = TimeAxisUtils.PrevFracIndex(
            t,
            sampleRate,
            alignLevel);

        // Assert
        Assert.Equal(ans, result);
    }

    [Theory]
    [InlineData(0e-9, 0)]
    [InlineData(0.5e-9, 0)]
    [InlineData(1e-9, 1)]
    [InlineData(10.5e-9, 10)]
    [InlineData(-10.5e-9, -10)]
    public void ClosestIndex_Normal_EqualExpected(double t, int ans)
    {
        // Arrange
        var sampleRate = SampleRate;

        // Act
        var result = TimeAxisUtils.ClosestIndex(
            t,
            sampleRate);

        // Assert
        Assert.Equal(ans, result);
    }

    [Theory]
    [InlineData(0, -1, 0)]
    [InlineData(0.25e-9, -1, 0)]
    [InlineData(0.5e-9, -1, 0.5)]
    [InlineData(0.8e-9, -1, 1)]
    [InlineData(10.8e-9, -1, 11)]
    [InlineData(-10.8e-9, -1, -11)]
    [InlineData(0, -2, 0)]
    [InlineData(0.25e-9, -2, 0.25)]
    [InlineData(0.5e-9, -2, 0.5)]
    [InlineData(0.8e-9, -2, 0.75)]
    [InlineData(10.8e-9, -2, 10.75)]
    [InlineData(-10.8e-9, -2, -10.75)]
    public void ClosestFracIndex_Normal_EqualExpected(double t, int alignLevel, double ans)
    {
        // Arrange
        var sampleRate = SampleRate;

        // Act
        var result = TimeAxisUtils.ClosestFracIndex(
            t,
            sampleRate,
            alignLevel);

        // Assert
        Assert.Equal(ans, result);
    }

    [Theory]
    [InlineData(0e-9, 10.5e-9, 0, 11)]
    [InlineData(0.5e-9, -10.5e-9, -10, 1)]
    public void GetIndexRange_Normal_EqualExpected(double tStart, double tEnd, int ansStart, int ansEnd)
    {
        // Arrange
        var sampleRate = SampleRate;

        // Act
        var (start, end) = TimeAxisUtils.GetIndexRange(
            tStart,
            tEnd,
            sampleRate);

        // Assert
        Assert.Equal(ansStart, start);
        Assert.Equal(ansEnd, end);
    }
}
