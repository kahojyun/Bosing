using System.Diagnostics;

using CommunityToolkit.Diagnostics;

using MessagePack;

namespace Qynit.PulseGen.Server.Models;


[Union(0, typeof(PlayElementDto))]
[Union(1, typeof(ShiftPhaseElementDto))]
[Union(2, typeof(SetPhaseElementDto))]
[Union(3, typeof(ShiftFrequencyElementDto))]
[Union(4, typeof(SetFrequencyElementDto))]
[Union(5, typeof(SwapPhaseElementDto))]
[Union(6, typeof(BarrierElementDto))]
[Union(7, typeof(RepeatElementDto))]
[Union(8, typeof(StackScheduleDto))]
[Union(9, typeof(AbsoluteScheduleDto))]
[Union(10, typeof(GridScheduleDto))]
[MessagePackObject]
public abstract class ScheduleElementDto
{
    [Key(0)]
    public (double, double) MarginData { get; init; }
    [Key(1)]
    public Alignment Alignment { get; init; }
    [Key(2)]
    public bool IsVisible { get; init; }
    [Key(3)]
    public double? Duration { get; init; }
    [Key(4)]
    public double MaxDuration { get; init; }
    [Key(5)]
    public double MinDuration { get; init; }

    [IgnoreMember]
    public Thickness Margin => new(MarginData.Item1, MarginData.Item2);
    public abstract ScheduleElement GetScheduleElement(ScheduleRequest request);
}


[MessagePackObject]
public sealed class PlayElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId { get; init; }
    [Key(7)]
    public double Amplitude { get; init; }
    [Key(8)]
    public int ShapeId { get; init; }
    [Key(9)]
    public double Width { get; init; }
    [Key(10)]
    public double Plateau { get; init; }
    [Key(11)]
    public double DragCoefficient { get; init; }
    [Key(12)]
    public double Frequency { get; init; }
    [Key(13)]
    public double Phase { get; init; }
    [Key(14)]
    public bool FlexiblePlateau { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        var shapes = request.ShapeTable;
        Debug.Assert(shapes is not null);
        var pulseShape = ShapeId == -1 ? null : shapes[ShapeId].GetPulseShape();
        var envelope = new Envelope(pulseShape, Width, Plateau);
        return new PlayElement(ChannelId, envelope, Frequency, Phase, Amplitude, DragCoefficient)
        {
            FlexiblePlateau = FlexiblePlateau,
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
    }
}


[MessagePackObject]
public sealed class ShiftPhaseElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId { get; init; }
    [Key(7)]
    public double DeltaPhase { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        return new ShiftPhaseElement(ChannelId, DeltaPhase)
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
    }
}


[MessagePackObject]
public sealed class SetPhaseElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId { get; init; }
    [Key(7)]
    public double Phase { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        return new SetPhaseElement(ChannelId, Phase)
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
    }
}


[MessagePackObject]
public sealed class ShiftFrequencyElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId { get; init; }
    [Key(7)]
    public double DeltaFrequency { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        return new ShiftFrequencyElement(ChannelId, DeltaFrequency)
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
    }
}


[MessagePackObject]
public sealed class SetFrequencyElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId { get; init; }
    [Key(7)]
    public double Frequency { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        return new SetFrequencyElement(ChannelId, Frequency)
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
    }
}


[MessagePackObject]
public sealed class SwapPhaseElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId1 { get; init; }
    [Key(7)]
    public int ChannelId2 { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        return new SwapPhaseElement(ChannelId1, ChannelId2)
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
    }
}


[MessagePackObject]
public sealed class BarrierElementDto : ScheduleElementDto
{
    [Key(6)]
    public ISet<int>? ChannelIds { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(ChannelIds);
        return new BarrierElement(ChannelIds)
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
    }
}


[MessagePackObject]
public sealed class RepeatElementDto : ScheduleElementDto
{
    [Key(6)]
    public ScheduleElementDto? Element { get; init; }
    [Key(7)]
    public int Count { get; init; }
    [Key(8)]
    public double Spacing { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(Element);
        var element = Element.GetScheduleElement(request);
        return new RepeatElement(element, Count)
        {
            Spacing = Spacing,
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
    }
}


[MessagePackObject]
public sealed class StackScheduleDto : ScheduleElementDto
{
    [Key(6)]
    public IList<ScheduleElementDto>? Elements { get; init; }
    [Key(7)]
    public ArrangeOption ArrangeOption { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(Elements);
        var result = new StackSchedule()
        {
            ArrangeOption = ArrangeOption,
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
        foreach (var element in Elements)
        {
            result.Add(element.GetScheduleElement(request));
        }
        return result;
    }
}


[MessagePackObject]
public sealed class AbsoluteScheduleDto : ScheduleElementDto
{
    [Key(6)]
    public IList<(double Time, ScheduleElementDto Element)>? Elements { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(Elements);
        var result = new AbsoluteSchedule()
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
        foreach (var (time, element) in Elements)
        {
            result.Add(element.GetScheduleElement(request), time);
        }
        return result;
    }
}


[MessagePackObject]
public sealed class GridScheduleDto : ScheduleElementDto
{
    [Key(6)]
    public IList<(int Column, int Span, ScheduleElementDto Element)>? Elements { get; init; }
    [Key(7)]
    public IList<(double Value, GridLengthUnit Unit)>? Columns { get; init; }

    public override ScheduleElement GetScheduleElement(ScheduleRequest request)
    {
        Guard.IsNotNull(Elements);
        Guard.IsNotNull(Columns);
        var result = new GridSchedule()
        {
            Margin = Margin,
            Alignment = Alignment,
            IsVisible = IsVisible,
            Duration = Duration,
            MaxDuration = MaxDuration,
            MinDuration = MinDuration,
        };
        foreach (var (column, span, element) in Elements)
        {
            result.Add(element.GetScheduleElement(request), column, span);
        }
        foreach (var (value, unit) in Columns)
        {
            result.AddColumn(new(value, unit));
        }
        return result;
    }
}
