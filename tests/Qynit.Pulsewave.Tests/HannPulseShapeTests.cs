namespace Qynit.Pulsewave.Tests
{
    public class HannPulseShapeTests
    {
        [Theory]
        [InlineData(-1, 0)]
        [InlineData(1, 0)]
        [InlineData(0, 1)]
        [InlineData(0.25, 0.5)]
        [InlineData(-0.25, 0.5)]
        public void SampleAt_Normal_Equal(double x, double ans)
        {
            // Arrange
            var hannPulseShape = new HannPulseShape();

            // Act
            var result = hannPulseShape.SampleAt(
                x);

            // Assert
            var tolerance = 1e-9;
            Assert.Equal(ans, result.Real, tolerance);
            Assert.Equal(0, result.Imaginary, tolerance);
        }
    }
}
