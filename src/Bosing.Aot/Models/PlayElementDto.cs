using System.Diagnostics;

using MessagePack;

using Bosing.Schedules;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class PlayElementDto : ScheduleElementDto
{
    [Key(6)]
    public int ChannelId { get; set; }
    [Key(7)]
    public double Amplitude { get; set; }
    [Key(8)]
    public int ShapeId { get; set; }
    [Key(9)]
    public double Width { get; set; }
    [Key(10)]
    public double Plateau { get; set; }
    [Key(11)]
    public double DragCoefficient { get; set; }
    [Key(12)]
    public double Frequency { get; set; }
    [Key(13)]
    public double Phase { get; set; }
    [Key(14)]
    public bool FlexiblePlateau { get; set; }

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
            BosingOptions = request.Options?.GetOptions(),
        };
    }
}
