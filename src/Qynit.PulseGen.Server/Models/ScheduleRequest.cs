using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed class ScheduleRequest
{
    [Key(0)]
    public IList<ChannelInfo>? ChannelTable { get; init; }
    [Key(1)]
    public IList<ShapeInfo>? ShapeTable { get; init; }
    [Key(2)]
    public ScheduleElementDto? Schedule { get; init; }
}
