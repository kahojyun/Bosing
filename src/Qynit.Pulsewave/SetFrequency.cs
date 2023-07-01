namespace Qynit.Pulsewave;
public sealed record SetFrequency(double Frequency, double ReferenceTime, Channel Channel) : Instruction;
