using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[MessagePackObject]
public sealed record PlayDto(
    [property: Key(0)] double Time,
    [property: Key(1)] int ChannelId,
    [property: Key(2)] int ShapeId,
    [property: Key(3)] double Width,
    [property: Key(4)] double Plateau,
    [property: Key(5)] double FrequencyShift,
    [property: Key(6)] double PhaseShift,
    [property: Key(7)] double Amplitude,
    [property: Key(8)] double DragCoefficient) : InstructionDto;
