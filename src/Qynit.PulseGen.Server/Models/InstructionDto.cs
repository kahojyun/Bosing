using MessagePack;

namespace Qynit.PulseGen.Server.Models;

[Union(0, typeof(PlayDto))]
[Union(1, typeof(ShiftPhaseDto))]
[Union(2, typeof(SetPhaseDto))]
[Union(3, typeof(ShiftFrequencyDto))]
[Union(4, typeof(SetFrequencyDto))]
[Union(5, typeof(SwapPhaseDto))]
public abstract record InstructionDto;
