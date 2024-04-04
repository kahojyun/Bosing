using System.Numerics;

namespace Bosing;
public readonly record struct SignalFilter<T> where T : INumber<T>
{
    public BiquadChain<T> BiquadChain { get; init; }
    public FirCoefficients<T> Fir { get; init; }
    public SignalFilter(BiquadChain<T> biquadChain, FirCoefficients<T> fir)
    {
        BiquadChain = biquadChain;
        Fir = fir;
    }
    public SignalFilter(IEnumerable<BiquadCoefficients<T>> biquads, IEnumerable<T> fir)
    {
        BiquadChain = new(biquads);
        Fir = new(fir);
    }
    public static SignalFilter<T> Empty => new(BiquadChain<T>.Empty, FirCoefficients<T>.Empty);
    public static SignalFilter<T> Concat(SignalFilter<T> a, SignalFilter<T> b)
    {
        return new SignalFilter<T>(BiquadChain<T>.Concat(a.BiquadChain, b.BiquadChain), FirCoefficients<T>.Concat(a.Fir, b.Fir));
    }
    public void Filter<TData>(Span<TData> signal) where TData : unmanaged, INumber<TData>
    {
        BiquadChain.Filter(signal);
        Fir.Filter(signal);
    }
}
