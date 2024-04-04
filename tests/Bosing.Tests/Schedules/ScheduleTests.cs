using Bosing.Schedules;

namespace Bosing.Tests.Schedules;
public class ScheduleTests
{
    private class FakeSchedule : Schedule
    {
        public void Add(ScheduleElement scheduleElement)
        {
            Children.Add(scheduleElement);
        }

        protected override double ArrangeOverride(double time, double finalDuration)
        {
            throw new NotImplementedException();
        }

        protected override double MeasureOverride(double maxDuration)
        {
            throw new NotImplementedException();
        }
    }

    [Fact]
    public void Schedule_Channels()
    {
        var schedule = new FakeSchedule()
        {
            new FakeScheduleElement([0, 1]),
            new FakeScheduleElement([2, 1]),
        };
        Assert.Equal([0, 1, 2], schedule.Channels.Order());
    }

    [Fact]
    public void Render_WithChildrenTime()
    {
        var renderTimes = new List<double>();
        var children = new[]
        {
            new FakeScheduleElement() { RenderCallback = renderTimes.Add },
            new FakeScheduleElement(2, 2) { RenderCallback = renderTimes.Add },
        };
        var schedule = new AbsoluteSchedule
        {
            { children[0], 1 },
            { children[1], 3 },
        };
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        schedule.Render(0, new PhaseTrackingTransform());
        Assert.Equal([1.0, 3.0], renderTimes);
    }
}
