namespace Qynit.Pulsewave;
public record SetPhase(double Phase, Channel Channel) : Instruction(nameof(SetPhase), new[] { Channel });
