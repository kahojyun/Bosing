namespace Qynit.Pulsewave;
public record Play(
    IPulseShape PulseShape,
    double TStart,
    double Width,
    double Plateau,
    double Amplitude,
    double Frequency,
    double Phase,
    Channel Channel) : Instruction(nameof(Play), new[] { Channel });
