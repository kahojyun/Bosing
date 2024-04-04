using Bosing.Schedules;

namespace Bosing.Tests.Schedules;
public class ScheduleElementTests
{
    [Fact]
    public void Duration_Validation()
    {
        var element = new FakeScheduleElement(0, 0);
        Assert.Throws<ArgumentOutOfRangeException>(() => element.Duration = -1);
        Assert.Throws<ArgumentOutOfRangeException>(() => element.Duration = double.NaN);
        Assert.Throws<ArgumentOutOfRangeException>(() => element.MinDuration = -1);
        Assert.Throws<ArgumentOutOfRangeException>(() => element.MinDuration = double.NaN);
        Assert.Throws<ArgumentOutOfRangeException>(() => element.MaxDuration = -1);
        Assert.Throws<ArgumentOutOfRangeException>(() => element.MaxDuration = double.NaN);
    }

    [Theory]
    [InlineData(0, 0, 0)]
    [InlineData(0, 1, 1)]
    [InlineData(1, 1, 3)]
    [InlineData(-1, 1, 0)]
    public void Measure_Unconstrained(double margin, double innerDesired, double expected)
    {
        var element = new FakeScheduleElement(innerDesired, 0) { Margin = new Thickness(margin) };
        element.Measure(double.PositiveInfinity);
        Assert.Equal(expected, element.DesiredDuration);
        Assert.Equal(expected, element.UnclippedDesiredDuration);
    }

    [Theory]
    [InlineData(2, 0, 0, 0, 0)]
    [InlineData(2, 0, 1, 1, 1)]
    [InlineData(2, 1, 1, 2, 3)]
    [InlineData(2, -1, 1, 0, 0)]
    public void Measure_AvailableDurationLimit_Clipped(double availableDuration, double margin, double innerDesired, double expected, double expectedUnclipped)
    {
        var element = new FakeScheduleElement(innerDesired, 0) { Margin = new Thickness(margin) };
        element.Measure(availableDuration);
        Assert.Equal(expected, element.DesiredDuration);
        Assert.Equal(expectedUnclipped, element.UnclippedDesiredDuration);
    }

    [Theory]
    [InlineData(1, 0, double.PositiveInfinity, 1, 0)]
    [InlineData(2, 0, 1, 1, 0)]
    [InlineData(1, 2, 1, 2, 0)]
    public void Measure_WithRequest_Clamped(double durationRequest, double minDuration, double maxDuration, double expected, double expectedUnclipped)
    {
        var element = new FakeScheduleElement(0, 0)
        {
            Duration = durationRequest,
            MinDuration = minDuration,
            MaxDuration = maxDuration,
        };
        element.Measure(double.PositiveInfinity);
        Assert.Equal(expected, element.DesiredDuration);
        Assert.Equal(expectedUnclipped, element.UnclippedDesiredDuration);
    }

    [Theory]
    [InlineData(0.5, 1, 0, double.PositiveInfinity, 0.5)]
    [InlineData(0.5, 2, 0, 1, 0.5)]
    [InlineData(0.5, 1, 2, 1, 0.5)]
    public void Measure_WithRequestAvailableDurationLimit_Clipped(double availableDuration, double durationRequest, double minDuration, double maxDuration, double expected)
    {
        var element = new FakeScheduleElement(0, 0)
        {
            Duration = durationRequest,
            MinDuration = minDuration,
            MaxDuration = maxDuration,
        };
        element.Measure(availableDuration);
        Assert.Equal(expected, element.DesiredDuration);
    }

    [Fact]
    public void Arrange_NotMeasured_Throw()
    {
        var element = new FakeScheduleElement(0, 0);
        Assert.Throws<InvalidOperationException>(() => element.Arrange(0, 0));
    }

    [Fact]
    public void Arrange_SmallerThanMeasured_Throw()
    {
        var element = new FakeScheduleElement(1, 0);
        element.Measure(double.PositiveInfinity);
        Assert.Throws<InvalidOperationException>(() => element.Arrange(0, 0));
    }

    [Fact]
    public void Arrange_RequestSmallerThanMeasured_Throw()
    {
        var element = new FakeScheduleElement(1, 0)
        {
            Duration = 0
        };
        element.Measure(double.PositiveInfinity);
        Assert.Throws<InvalidOperationException>(() => element.Arrange(0, 2));
    }

    [Theory]
    [InlineData(0, 0)]
    [InlineData(1, 1)]
    public void ArrangeDuration_NoMargin(double arrangeResult, double expected)
    {
        var element = new FakeScheduleElement(arrangeResult, arrangeResult);
        element.Measure(double.PositiveInfinity);
        var measureResult = element.DesiredDuration;
        element.Arrange(0, measureResult!.Value);
        Assert.Equal(expected, element.ActualDuration);
    }

    [Theory]
    [InlineData(-1, -1)]
    [InlineData(0, 0)]
    [InlineData(1, 1)]
    public void ArrangeTime_NoMargin(double arrangeTime, double expected)
    {
        var element = new FakeScheduleElement(0, 0);
        element.Measure(double.PositiveInfinity);
        var measureResult = element.DesiredDuration;
        element.Arrange(arrangeTime, measureResult!.Value);
        Assert.Equal(expected, element.ActualTime);
    }

    [Theory]
    [InlineData(1, 2, 2)]
    [InlineData(-1, 2, 2)]
    public void ArrangeDuration_WithMargin(double margin, double arrangeResult, double expected)
    {
        var element = new FakeScheduleElement(arrangeResult, arrangeResult)
        {
            Margin = new Thickness(margin)
        };
        element.Measure(double.PositiveInfinity);
        var measureResult = element.DesiredDuration;
        element.Arrange(0, measureResult!.Value);
        Assert.Equal(expected, element.ActualDuration);
    }

    [Theory]
    [InlineData(1, 1, 2)]
    [InlineData(-1, 1, 0)]
    public void ArrangeTime_WithMargin(double margin, double arrangeTime, double expected)
    {
        var element = new FakeScheduleElement(0, 0)
        {
            Margin = new Thickness(margin)
        };
        element.Measure(double.PositiveInfinity);
        var measureResult = element.DesiredDuration;
        element.Arrange(arrangeTime, measureResult!.Value);
        Assert.Equal(expected, element.ActualTime);
    }

    [Fact]
    public void Render_NotArranged_Throw()
    {
        var element = new FakeScheduleElement(0, 0);
        element.Measure(double.PositiveInfinity);
        Assert.Throws<InvalidOperationException>(() => element.Render(0, new PhaseTrackingTransform()));
    }

    [Theory]
    [InlineData(0, 0, 0)]
    [InlineData(0, 1, 1)]
    [InlineData(1, 1, 2)]
    public void Render_WithMargin(double time, double margin, double expected)
    {
        var renderTime = double.NaN;
        var element = new FakeScheduleElement(0, 0)
        {
            Margin = new Thickness(margin),
            RenderCallback = x => renderTime = x,
        };
        element.Measure(double.PositiveInfinity);
        var measureResult = element.DesiredDuration;
        element.Arrange(0, measureResult!.Value);
        element.Render(time, new PhaseTrackingTransform());
        Assert.Equal(expected, renderTime);
    }
}
