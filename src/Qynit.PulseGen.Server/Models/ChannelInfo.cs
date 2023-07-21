using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed record ChannelInfo(
    [property: Key(0)] string Name,
    [property: Key(1)] double BaseFrequency,
    [property: Key(2)] double SampleRate,
    [property: Key(3)] double Delay,
    [property: Key(4)] int Length,
    [property: Key(5)] int AlignLevel);
