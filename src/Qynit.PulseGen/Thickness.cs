namespace Qynit.PulseGen;
public record struct Thickness(double Start, double End)
{
    public readonly double Total => Start + End;
}
