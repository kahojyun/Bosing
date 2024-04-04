using Bosing.Schedules;

namespace Bosing.Tests.Schedules;

public class RepeatElementTests
{
    [Fact]
    public void Measure()
    {
        var schedule = new RepeatElement(new FakeScheduleElement(1, 1), 3) { Spacing = 1 };
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(5, schedule.DesiredDuration);
    }

    [Fact]
    public void Arrange()
    {
        var child = new FakeScheduleElement(1, 1);
        var schedule = new RepeatElement(child, 3) { Spacing = 1 };
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(5, schedule.ActualDuration);
        Assert.Equal(0, child.ActualTime);
        Assert.Equal(1, child.ActualDuration);
    }

    [Fact]
    public void Render()
    {
        var renderTimes = new List<double>();
        var child = new FakeScheduleElement(1, 1) { RenderCallback = renderTimes.Add };
        var schedule = new RepeatElement(child, 3) { Spacing = 1 };
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        schedule.Render(0, new PhaseTrackingTransform());
        Assert.Equal([0.0, 2.0, 4.0], renderTimes);
    }
}
