namespace Qynit.PulseGen;
public sealed record Play(
    IPulseShape PulseShape,
    double TStart,
    double Width,
    double Plateau,
    double Amplitude,
    double DragCoefficient,
    double Frequency,
    double Phase,
    Channel Channel) : Instruction;
