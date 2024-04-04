namespace Bosing.Tests;

public class MathUtilsTests
{
    [Theory]
    [InlineData(-1, 0, 1, 0)]
    [InlineData(2, 0, 1, 1)]
    [InlineData(0.5, 0, 1, 0.5)]
    [InlineData(0, 0, -1, 0)]
    [InlineData(1, 0, -1, 0)]
    [InlineData(-1, 0, -1, 0)]
    public void Clamp_StateUnderTest_ExpectedBehavior(double value, double min, double max, double expected)
    {
        var result = MathUtils.Clamp(value, min, max);
        Assert.Equal(expected, result);
    }
}
