namespace Qynit.Pulsewave;
public record ShiftFrequency(double Frequency, double ReferenceTime, Channel Channel) : Instruction(nameof(ShiftFrequency), new[] { Channel });
