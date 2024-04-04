using System.Numerics;

namespace Bosing;

internal struct BiquadFilter<T>(BiquadCoefficients<T> coefficients) where T : INumber<T>
{
    public BiquadCoefficients<T> Coefficients { get; init; } = coefficients;
    private T _s0 = T.Zero;
    private T _s1 = T.Zero;

    public T Transform(T input)
    {
        var output = Coefficients.B0 * input + _s0;
        _s0 = Coefficients.B1 * input + _s1 - Coefficients.A1 * output;
        _s1 = Coefficients.B2 * input - Coefficients.A2 * output;
        return output;
    }
}
