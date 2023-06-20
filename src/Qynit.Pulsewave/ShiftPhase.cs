namespace Qynit.Pulsewave;
public record ShiftPhase(double Phase, Channel Channel) : Instruction(nameof(ShiftPhase), new[] { Channel });
