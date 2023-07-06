using MessagePack;

namespace Qynit.PulseGen.Server;

[MessagePackObject]
public sealed record SetFrequency(
    [property: Key(0)] double Time,
    [property: Key(1)] int ChannelId,
    [property: Key(2)] double Frequency) : Instruction;
