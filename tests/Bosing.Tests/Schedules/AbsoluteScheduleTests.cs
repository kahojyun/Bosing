using Bosing.Schedules;

namespace Bosing.Tests.Schedules;

public class AbsoluteScheduleTests
{
    [Fact]
    public void AbsoluteSchedule_Default()
    {
        var schedule = new AbsoluteSchedule();
        Assert.Equal(Alignment.Stretch, schedule.Alignment);
    }

    [Fact]
    public void Add_ChildParent()
    {
        var child = new FakeScheduleElement();
        var schedule = new AbsoluteSchedule
        {
            child
        };
        Assert.Same(schedule, child.Parent);
    }

    [Theory]
    [InlineData(double.NaN)]
    [InlineData(double.NegativeInfinity)]
    [InlineData(double.PositiveInfinity)]
    [InlineData(-1)]
    public void Add_IllegalTime_Throw(double time)
    {
        var schedule = new AbsoluteSchedule();
        Assert.Throws<ArgumentOutOfRangeException>(() => schedule.Add(new FakeScheduleElement(), time));
    }

    [Fact]
    public void Measure_Empty()
    {
        var schedule = new AbsoluteSchedule();
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(0, schedule.DesiredDuration);
    }

    [Fact]
    public void Measure_WithChildren()
    {
        var schedule = new AbsoluteSchedule
        {
            new FakeScheduleElement(1, 1),
            new FakeScheduleElement(3, 3),
        };
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(3, schedule.DesiredDuration);
    }

    [Fact]
    public void Measure_WithChildrenTime()
    {
        var schedule = new AbsoluteSchedule
        {
            { new FakeScheduleElement(), 1 },
            { new FakeScheduleElement(), 3 },
        };
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(3, schedule.DesiredDuration);
    }

    [Fact]
    public void Arrange_Empty()
    {
        var schedule = new AbsoluteSchedule();
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(0, schedule.ActualDuration);
    }

    [Fact]
    public void Arrange_WithChildren()
    {
        var schedule = new AbsoluteSchedule
        {
            new FakeScheduleElement(1, 1),
            new FakeScheduleElement(3, 3),
        };
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(3, schedule.ActualDuration);
    }

    [Fact]
    public void Arrange_WithChildrenTime()
    {
        var schedule = new AbsoluteSchedule
        {
            { new FakeScheduleElement(), 1 },
            { new FakeScheduleElement(), 3 },
        };
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(3, schedule.ActualDuration);
    }

    [Fact]
    public void Arrange_LargeSpace()
    {
        var schedule = new AbsoluteSchedule();
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 1);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(1, schedule.ActualDuration);
    }
}
