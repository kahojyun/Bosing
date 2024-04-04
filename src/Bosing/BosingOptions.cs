namespace Bosing;
public record BosingOptions
{
    /// <summary>
    /// Tolerance for comparing times.
    /// </summary>
    public double TimeTolerance { get; init; } = 1e-12;
    /// <summary>
    /// Tolerance for comparing phases.
    /// </summary>
    public double PhaseTolerance { get; init; } = 1e-4;
    /// <summary>
    /// Tolerance for comparing amplitudes.
    /// </summary>
    public double AmpTolerance { get; init; } = 0.1 / ushort.MaxValue;
    public bool AllowOversize { get; init; } = false;

    public static BosingOptions Default { get; } = new();
}
