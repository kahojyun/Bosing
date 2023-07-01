namespace Qynit.Pulsewave;
public sealed record ShiftFrequency(double Frequency, double ReferenceTime, Channel Channel) : Instruction;
