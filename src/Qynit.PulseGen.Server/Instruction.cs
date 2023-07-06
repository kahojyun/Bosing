using MessagePack;

namespace Qynit.PulseGen.Server;

[Union(0, typeof(Play))]
[Union(1, typeof(ShiftPhase))]
[Union(2, typeof(SetPhase))]
[Union(3, typeof(ShiftFrequency))]
[Union(4, typeof(SetFrequency))]
[Union(5, typeof(SwapPhase))]
public abstract record Instruction;
