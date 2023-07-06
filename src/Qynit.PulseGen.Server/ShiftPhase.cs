using MessagePack;

namespace Qynit.PulseGen.Server;

[MessagePackObject]
public sealed record ShiftPhase(
    [property: Key(0)] int ChannelId,
    [property: Key(1)] double Phase) : Instruction;
