using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed record SwapPhaseDto(
    [property: Key(0)] double Time,
    [property: Key(1)] int ChannelId1,
    [property: Key(2)] int ChannelId2) : InstructionDto;
