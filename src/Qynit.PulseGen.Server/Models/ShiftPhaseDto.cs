using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed record ShiftPhaseDto(
    [property: Key(0)] int ChannelId,
    [property: Key(1)] double Phase) : InstructionDto;
