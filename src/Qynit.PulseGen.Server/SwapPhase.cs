using MessagePack;

namespace Qynit.PulseGen.Server;

[MessagePackObject]
public sealed record SwapPhase(
    [property: Key(0)] double Time,
    [property: Key(1)] int ChannelId1,
    [property: Key(2)] int ChannelId2) : Instruction;
