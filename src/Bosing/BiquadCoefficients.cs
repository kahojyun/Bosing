using System.Numerics;

namespace Bosing;
public readonly record struct BiquadCoefficients<T> where T : INumber<T>
{
    public T B0 { get; init; }
    public T B1 { get; init; }
    public T B2 { get; init; }
    public T A1 { get; init; }
    public T A2 { get; init; }
    public static BiquadCoefficients<T> Identity => new()
    {
        B0 = T.One,
        B1 = T.Zero,
        B2 = T.Zero,
        A1 = T.Zero,
        A2 = T.Zero,
    };
    public BiquadCoefficients<TTo> Cast<TTo>() where TTo : INumber<TTo>
    {
        return new()
        {
            B0 = TTo.CreateChecked(B0),
            B1 = TTo.CreateChecked(B1),
            B2 = TTo.CreateChecked(B2),
            A1 = TTo.CreateChecked(A1),
            A2 = TTo.CreateChecked(A2),
        };
    }
}
