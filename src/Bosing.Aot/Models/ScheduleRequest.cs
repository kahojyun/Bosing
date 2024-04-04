using MessagePack;

namespace Bosing.Aot.Models;

[MessagePackObject]
public sealed class ScheduleRequest
{
    [Key(0)]
    public IList<ChannelInfo>? ChannelTable { get; set; }
    [Key(1)]
    public IList<ShapeInfo>? ShapeTable { get; set; }
    [Key(2)]
    public ScheduleElementDto? Schedule { get; set; }
    [Key(3)]
    public OptionsDto? Options { get; set; }
}
