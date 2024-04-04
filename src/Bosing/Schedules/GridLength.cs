using CommunityToolkit.Diagnostics;

namespace Bosing.Schedules;
public readonly record struct GridLength(double Value, GridLengthUnit Unit)
{
    public static GridLength Auto => new(double.NaN, GridLengthUnit.Auto);

    public static GridLength Star(double value)
    {
        Guard.IsGreaterThan(value, 0);
        return new(value, GridLengthUnit.Star);
    }

    public static GridLength Absolute(double value)
    {
        Guard.IsGreaterThanOrEqualTo(value, 0);
        return new(value, GridLengthUnit.Second);
    }

    public bool IsAuto => Unit == GridLengthUnit.Auto;
    public bool IsStar => Unit == GridLengthUnit.Star;
    public bool IsAbsolute => Unit == GridLengthUnit.Second;
    public bool IsValid => IsAuto || IsStar && Value > 0 || IsAbsolute && Value >= 0;

    public override string ToString()
    {
        return Unit switch
        {
            GridLengthUnit.Auto => "Auto",
            GridLengthUnit.Star => $"{Value}*",
            GridLengthUnit.Second => $"{Value}s",
            _ => $$"""GridLength { Value = {{Value}}, Unit = {{Unit}} }""",
        };
    }
}
