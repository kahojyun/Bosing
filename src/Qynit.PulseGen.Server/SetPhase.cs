using MessagePack;

namespace Qynit.PulseGen.Server;

[MessagePackObject]
public sealed record SetPhase(
    [property: Key(0)] double Time,
    [property: Key(1)] int ChannelId,
    [property: Key(2)] double Phase) : Instruction;
