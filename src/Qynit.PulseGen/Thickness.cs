namespace Qynit.PulseGen;
public record struct Thickness(double Start, double End)
{
    public Thickness(double value) : this(value, value) { }

    public readonly double Total => Start + End;
}
