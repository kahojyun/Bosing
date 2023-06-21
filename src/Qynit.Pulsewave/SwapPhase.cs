namespace Qynit.Pulsewave;
public record SwapPhase(double ReferenceTime, Channel Channel1, Channel Channel2) : Instruction(nameof(SwapPhase), new[] { Channel1, Channel2 });
