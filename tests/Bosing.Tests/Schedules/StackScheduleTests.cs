using Bosing.Schedules;

namespace Bosing.Tests.Schedules;

public class StackScheduleTests
{
    [Fact]
    public void StackSchedule_Default()
    {
        var schedule = new StackSchedule();
        Assert.Equal(Alignment.Stretch, schedule.Alignment);
    }

    [Fact]
    public void Measure_Empty()
    {
        var schedule = new StackSchedule();
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(0, schedule.DesiredDuration);
    }

    [Fact]
    public void Measure_ChildrenNoChannel()
    {
        var schedule = new StackSchedule
        {
            new FakeScheduleElement(1, 1),
            new FakeScheduleElement(3, 3),
        };
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(4, schedule.DesiredDuration);
    }

    [Fact]
    public void Measure_ChildrenNoSync()
    {
        var schedule = new StackSchedule
        {
            new FakeScheduleElement(1, 1, [ 0 ]),
            new FakeScheduleElement(3, 3, [ 1 ]),
            new FakeScheduleElement(1, 1, [ 1 ]),
            new FakeScheduleElement(3, 3, [ 0 ]),
        };
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(4, schedule.DesiredDuration);
    }

    [Fact]
    public void Measure_ChildrenSync()
    {
        var schedule = new StackSchedule
        {
            new FakeScheduleElement(1, 1, [ 0 ]),
            new FakeScheduleElement(3, 3, [ 1 ]),
            new FakeScheduleElement([ 0, 1 ]),
            new FakeScheduleElement(1, 1, [ 1 ]),
            new FakeScheduleElement(3, 3, [ 0 ]),
        };
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(6, schedule.DesiredDuration);
    }

    [Fact]
    public void Arrange_Empty()
    {
        var schedule = new StackSchedule();
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(0, schedule.ActualDuration);
    }

    [Fact]
    public void Arrange_EmptyLargeSpace()
    {
        var schedule = new StackSchedule();
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 1);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(1, schedule.ActualDuration);
    }

    [Fact]
    public void Arrange_ChildrenNoChannelEndToStart()
    {
        var children = new[]
        {
            new FakeScheduleElement(1, 1),
            new FakeScheduleElement(3, 3),
        };
        var schedule = new StackSchedule
        {
            children[0],
            children[1],
        };
        schedule.ArrangeOption = ArrangeOption.EndToStart;
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 10);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(10, schedule.ActualDuration);
        Assert.Equal(6, children[0].ActualTime);
        Assert.Equal(7, children[1].ActualTime);
    }

    [Fact]
    public void Arrange_ChildrenNoChannelStartToEnd()
    {
        var children = new[]
        {
            new FakeScheduleElement(1, 1),
            new FakeScheduleElement(3, 3),
        };
        var schedule = new StackSchedule
        {
            children[0],
            children[1],
        };
        schedule.ArrangeOption = ArrangeOption.StartToEnd;
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 10);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(10, schedule.ActualDuration);
        Assert.Equal(0, children[0].ActualTime);
        Assert.Equal(1, children[1].ActualTime);
    }

    [Fact]
    public void Arrange_ChildrenNoSyncEndToStart()
    {
        var children = new[]
        {
            new FakeScheduleElement(1, 1, [0]),
            new FakeScheduleElement(3, 3, [1]),
            new FakeScheduleElement(1, 1, [1]),
            new FakeScheduleElement(3, 3, [0]),
        };
        var schedule = new StackSchedule
        {
            children[0],
            children[1],
            children[2],
            children[3],
        };
        schedule.ArrangeOption = ArrangeOption.EndToStart;
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 10);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(10, schedule.ActualDuration);
        Assert.Equal(6, children[0].ActualTime);
        Assert.Equal(6, children[1].ActualTime);
        Assert.Equal(9, children[2].ActualTime);
        Assert.Equal(7, children[3].ActualTime);
    }

    [Fact]
    public void Arrange_ChildrenNoSyncStartToEnd()
    {
        var children = new[]
        {
            new FakeScheduleElement(1, 1, [0]),
            new FakeScheduleElement(3, 3, [1]),
            new FakeScheduleElement(1, 1, [1]),
            new FakeScheduleElement(3, 3, [0]),
        };
        var schedule = new StackSchedule
        {
            children[0],
            children[1],
            children[2],
            children[3],
        };
        schedule.ArrangeOption = ArrangeOption.StartToEnd;
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 10);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(10, schedule.ActualDuration);
        Assert.Equal(0, children[0].ActualTime);
        Assert.Equal(0, children[1].ActualTime);
        Assert.Equal(3, children[2].ActualTime);
        Assert.Equal(1, children[3].ActualTime);
    }

    [Fact]
    public void Arrange_ChildrenSyncEndToStart()
    {
        var children = new[]
        {
            new FakeScheduleElement(1, 1, [0]),
            new FakeScheduleElement(3, 3, [1]),
            new FakeScheduleElement([0, 1]),
            new FakeScheduleElement(1, 1, [1]),
            new FakeScheduleElement(3, 3, [0]),
        };
        var schedule = new StackSchedule
        {
            children[0],
            children[1],
            children[2],
            children[3],
            children[4],
        };
        schedule.ArrangeOption = ArrangeOption.EndToStart;
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 10);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(10, schedule.ActualDuration);
        Assert.Equal(6, children[0].ActualTime);
        Assert.Equal(4, children[1].ActualTime);
        Assert.Equal(7, children[2].ActualTime);
        Assert.Equal(9, children[3].ActualTime);
        Assert.Equal(7, children[4].ActualTime);
    }

    [Fact]
    public void Arrange_ChildrenSyncStartToEnd()
    {
        var children = new[]
        {
            new FakeScheduleElement(1, 1, [0]),
            new FakeScheduleElement(3, 3, [1]),
            new FakeScheduleElement([0, 1]),
            new FakeScheduleElement(1, 1, [1]),
            new FakeScheduleElement(3, 3, [0]),
        };
        var schedule = new StackSchedule
        {
            children[0],
            children[1],
            children[2],
            children[3],
            children[4],
        };
        schedule.ArrangeOption = ArrangeOption.StartToEnd;
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 10);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(10, schedule.ActualDuration);
        Assert.Equal(0, children[0].ActualTime);
        Assert.Equal(0, children[1].ActualTime);
        Assert.Equal(3, children[2].ActualTime);
        Assert.Equal(3, children[3].ActualTime);
        Assert.Equal(3, children[4].ActualTime);
    }

    [Fact]
    public void Arrange_ChildrenAlignment_Ignore()
    {
        var children = new[]
        {
            new FakeScheduleElement(1, 1, [0]) { Alignment = Alignment.Center },
            new FakeScheduleElement(3, 3, [1]) { Alignment = Alignment.Stretch },
            new FakeScheduleElement([0, 1]) { Alignment = Alignment.Start },
            new FakeScheduleElement(1, 1, [1]) { Alignment = Alignment.End },
            new FakeScheduleElement(3, 3, [0]),
        };
        var schedule = new StackSchedule
        {
            children[0],
            children[1],
            children[2],
            children[3],
            children[4],
        };
        schedule.ArrangeOption = ArrangeOption.EndToStart;
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 10);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(10, schedule.ActualDuration);
        Assert.Equal(6, children[0].ActualTime);
        Assert.Equal(4, children[1].ActualTime);
        Assert.Equal(7, children[2].ActualTime);
        Assert.Equal(9, children[3].ActualTime);
        Assert.Equal(7, children[4].ActualTime);
    }
}
