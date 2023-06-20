namespace Qynit.Pulsewave;
public record SetFrequency(double Frequency, double ReferenceTime, Channel Channel) : Instruction(nameof(SetFrequency), new[] { Channel });
