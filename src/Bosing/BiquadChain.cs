using System.Numerics;

namespace Bosing;

public readonly record struct BiquadChain<T> where T : INumber<T>
{
    private ValueArray<BiquadCoefficients<T>> Coefficients { get; init; }
    public BiquadChain(IEnumerable<BiquadCoefficients<T>> coefficients)
    {
        Coefficients = new(coefficients);
    }
    public static BiquadChain<T> Empty => new(Enumerable.Empty<BiquadCoefficients<T>>());
    public static BiquadChain<T> Concat(BiquadChain<T> a, BiquadChain<T> b)
    {
        var coefA = a.Coefficients.Data;
        var coefB = b.Coefficients.Data;
        return new BiquadChain<T>(coefA.Concat(coefB));
    }
    public void Filter<TData>(Span<TData> signal) where TData : unmanaged, INumber<TData>
    {
        if (Coefficients.Count == 0)
        {
            return;
        }
        Span<BiquadFilter<TData>> filters = stackalloc BiquadFilter<TData>[Coefficients.Count];
        for (var i = 0; i < filters.Length; i++)
        {
            filters[i] = new(Coefficients[i].Cast<TData>());
        }
        for (var i = 0; i < signal.Length; i++)
        {
            var sample = signal[i];
            for (var j = 0; j < filters.Length; j++)
            {
                sample = filters[j].Transform(sample);
            }
            signal[i] = sample;
        }
    }
}
