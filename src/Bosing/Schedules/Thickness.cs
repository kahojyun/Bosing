namespace Bosing.Schedules;
public readonly record struct Thickness(double Start, double End)
{
    public Thickness(double value) : this(value, value) { }

    public double Total => Start + End;
}
