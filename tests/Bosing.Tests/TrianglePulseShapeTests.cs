namespace Bosing.Tests;

public class TrianglePulseShapeTests
{
    [Theory]
    [InlineData(-1, 0)]
    [InlineData(1, 0)]
    [InlineData(0, 1)]
    [InlineData(0.25, 0.5)]
    [InlineData(-0.25, 0.5)]
    public void SampleAt_Double_Equal(double x, double ans)
    {
        // Arrange
        var shape = new TrianglePulseShape();

        // Act
        var (i, q) = shape.SampleAt(
            x);

        // Assert
        var tolerance = double.BitIncrement(ans) - ans;
        Assert.Equal(ans, i, tolerance);
        Assert.Equal(0, q, tolerance);
    }

    [Theory]
    [InlineData(-1, 0)]
    [InlineData(1, 0)]
    [InlineData(0, 1)]
    [InlineData(0.25, 0.5)]
    [InlineData(-0.25, 0.5)]
    public void SampleAt_Float_Equal(float x, float ans)
    {
        // Arrange
        var shape = new TrianglePulseShape();

        // Act
        var (i, q) = shape.SampleAt(
            x);

        // Assert
        var tolerance = float.BitIncrement(ans) - ans;
        Assert.Equal(ans, i, tolerance);
        Assert.Equal(0, q, tolerance);
    }
}
