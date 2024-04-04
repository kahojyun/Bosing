using Bosing.Schedules;

namespace Bosing.Tests.Schedules;

public class GridScheduleTests
{
    [Fact]
    public void GridSchedule_Default()
    {
        var schedule = new GridSchedule();
        Assert.Equal(Alignment.Stretch, schedule.Alignment);
    }

    [Fact]
    public void Measure_Empty()
    {
        var schedule = new GridSchedule();
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(0, schedule.DesiredDuration);
    }

    public static TheoryData<double, double, GridLength[]> Measure_Data => new()
    {
        { 1, 1, [ GridLength.Auto, GridLength.Auto ] },
        { 1, 1, [ GridLength.Auto, GridLength.Star(1) ] },
        { 1, 1, [ GridLength.Star(1), GridLength.Star(2) ] },
        { 2, 2, [ GridLength.Absolute(1), GridLength.Auto ] },
        { 2, 2, [ GridLength.Absolute(1), GridLength.Star(1) ] },
        { 2, 3, [ GridLength.Absolute(1), GridLength.Absolute(2) ] },
        { 4, 3, [ GridLength.Absolute(1), GridLength.Absolute(2) ] },
    };

    [Theory]
    [MemberData(nameof(Measure_Data))]
    public void Measure_Span(double duration, double desired, GridLength[] columns)
    {
        var schedule = new GridSchedule
        {
            { new FakeScheduleElement() { Duration = duration }, 0, columns.Length },
        };
        foreach (var col in columns)
        {
            schedule.AddColumn(col);
        }
        schedule.Measure(double.PositiveInfinity);
        Assert.Equal(desired, schedule.DesiredDuration);
    }

    [Fact]
    public void Arrange_Empty()
    {
        var schedule = new GridSchedule();
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        Assert.Equal(0, schedule.ActualTime);
        Assert.Equal(0, schedule.ActualDuration);
    }

    [Fact]
    public void Arrange_FixedSpan()
    {
        var children = new[]
        {
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.Stretch },
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.Stretch },
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.Stretch },
        };
        var schedule = new GridSchedule
        {
            children[0],
            { children[1], 1, 1 },
            { children[2], 0, 2 },
        };
        schedule.AddColumn(GridLength.Absolute(1));
        schedule.AddColumn(GridLength.Absolute(2));
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        Assert.Equal(0, children[0].ActualTime);
        Assert.Equal(1, children[0].ActualDuration);
        Assert.Equal(1, children[1].ActualTime);
        Assert.Equal(2, children[1].ActualDuration);
        Assert.Equal(0, children[2].ActualTime);
        Assert.Equal(3, children[2].ActualDuration);
    }

    public static TheoryData<GridLength, GridLength, double> Arrange_Span_Data => new()
    {
        { GridLength.Auto, GridLength.Auto, 3 },
        { GridLength.Star(1), GridLength.Star(2), 2 },
        { GridLength.Auto, GridLength.Star(1), 0 },
        { GridLength.Auto, GridLength.Absolute(2), 4 },
        { GridLength.Star(1), GridLength.Absolute(2), 4 },
    };

    [Theory]
    [MemberData(nameof(Arrange_Span_Data))]
    public void Arrange_Span(GridLength col1, GridLength col2, double splitTime)
    {
        var totalDuration = 6;
        var children = new[]
        {
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.Stretch },
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.Stretch },
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.Stretch, Duration = totalDuration },
        };
        var schedule = new GridSchedule
        {
            children[0],
            { children[1], 1, 1 },
            { children[2], 0, 2 },
        };
        schedule.AddColumn(col1);
        schedule.AddColumn(col2);
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        Assert.Equal(0, children[0].ActualTime);
        Assert.Equal(splitTime, children[0].ActualDuration);
        Assert.Equal(splitTime, children[1].ActualTime);
        Assert.Equal(totalDuration - splitTime, children[1].ActualDuration);
        Assert.Equal(0, children[2].ActualTime);
        Assert.Equal(totalDuration, children[2].ActualDuration);
    }

    public static TheoryData<Alignment, double[]> Arrange_Align_Data => new()
    {
        { Alignment.Start, [ 0.0, 2.0, 0.0 ] },
        { Alignment.Center, [ 0.5, 2.5, 1.5 ] },
        { Alignment.End, [ 1.0, 3.0, 3.0 ] },
        { Alignment.Stretch, [ 0.0, 2.0, 0.0 ] },
    };

    [Theory]
    [MemberData(nameof(Arrange_Align_Data))]
    public void Arrange_Align(Alignment alignment, double[] expected)
    {
        var children = new[]
        {
            new FakeScheduleElement() { Flexible = true, Alignment = alignment, Duration = 1 },
            new FakeScheduleElement() { Flexible = true, Alignment = alignment, Duration = 1 },
            new FakeScheduleElement() { Flexible = true, Alignment = alignment, Duration = 1 },
        };
        var schedule = new GridSchedule
        {
            children[0],
            { children[1], 1, 1 },
            { children[2], 0, 2 },
        };
        schedule.AddColumn(GridLength.Absolute(2));
        schedule.AddColumn(GridLength.Absolute(2));
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, schedule.DesiredDuration!.Value);
        Assert.Equal(expected, children.Select(x => x.ActualTime!.Value));
    }

    [Fact]
    public void Arrange_SameColumn()
    {
        var children = new[]
        {
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.Start, Duration = 1 },
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.Center, Duration = 1 },
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.End, Duration = 1 },
            new FakeScheduleElement() { Flexible = true, Alignment = Alignment.Stretch, Duration = 1 },
        };
        var schedule = new GridSchedule();
        foreach (var child in children)
        {
            schedule.Add(child);
        }
        schedule.Measure(double.PositiveInfinity);
        schedule.Arrange(0, 3);
        Assert.Equal([0.0, 1.0, 2.0, 0.0], children.Select(x => x.ActualTime!.Value));
    }
}
