using System.Numerics;

namespace Qynit.Pulsewave;

/// <summary>
/// Represents a pulse shape envelope.
/// </summary>
public interface IPulseShape
{
    /// <summary>
    /// Sample pulse shape envelope at x.
    /// </summary>
    /// <param name="x">x value in range -0.5 to -0.5</param>
    /// <returns>y value sampled at x</returns>
    Complex SampleAt(double x);
}
