namespace Qynit.Pulsewave.Tests;

public class MathUtilsTests
{
    [Theory]
    [InlineData(1.49, 0.2, 1.4)]
    [InlineData(1.51, 0.2, 1.6)]
    [InlineData(1.51, -0.2, 1.6)]
    public void MRound_Normal_Pass(double value, double multiple, double ans)
    {
        // Act
        var result = MathUtils.MRound(
            value,
            multiple);

        // Assert
        Assert.Equal(ans, result, Math.Abs(1e-6 * multiple));
    }

    [Theory]
    [InlineData(1.49, 0.2, 1.4)]
    [InlineData(1.51, 0.2, 1.4)]
    [InlineData(1.51, -0.2, 1.6)]
    public void MFloor_Normal_Pass(double value, double multiple, double ans)
    {
        // Act
        var result = MathUtils.MFloor(
            value,
            multiple);

        // Assert
        Assert.Equal(ans, result, Math.Abs(1e-6 * multiple));
    }

    [Theory]
    [InlineData(1.49, 0.2, 1.6)]
    [InlineData(1.51, 0.2, 1.6)]
    [InlineData(1.51, -0.2, 1.4)]
    public void MCeiling_Normal_Pass(double value, double multiple, double ans)
    {
        // Act
        var result = MathUtils.MCeiling(
            value,
            multiple);

        // Assert
        Assert.Equal(ans, result, Math.Abs(1e-6 * multiple));
    }
}
