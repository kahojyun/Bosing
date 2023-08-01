using System.Collections;
using System.Numerics;

namespace Qynit.PulseGen;
public readonly record struct FirCoefficients<T> : IReadOnlyList<T> where T : INumber<T>
{
    internal ValueArray<T> Coefficients { get; init; }
    public FirCoefficients(IEnumerable<T> coefficients)
    {
        Coefficients = new(coefficients);
    }
    public static FirCoefficients<T> Empty => new(Enumerable.Empty<T>());

    public int Count => Coefficients.Count;

    public T this[int index] => Coefficients[index];

    public FirCoefficients<TTo> Cast<TTo>() where TTo : INumber<TTo>
    {
        return new(Coefficients.Data.Select(TTo.CreateChecked));
    }

    public IEnumerator<T> GetEnumerator()
    {
        return Coefficients.GetEnumerator();
    }

    IEnumerator IEnumerable.GetEnumerator()
    {
        return ((IEnumerable)Coefficients).GetEnumerator();
    }

    public static FirCoefficients<T> Concat(FirCoefficients<T> a, FirCoefficients<T> b)
    {
        if (a.Count == 0)
        {
            return b;
        }
        if (b.Count == 0)
        {
            return a;
        }
        var coefA = a.Coefficients.Data;
        var coefB = b.Coefficients.Data;
        var newCoef = new T[coefA.Length + coefB.Length - 1];
        for (var i = 0; i < coefA.Length; i++)
        {
            for (var j = 0; j < coefB.Length; j++)
            {
                newCoef[i + j] += coefA[i] * coefB[j];
            }
        }
        return new FirCoefficients<T>(newCoef);
    }

    public void Filter<TData>(Span<TData> signal) where TData : unmanaged, INumber<TData>
    {
        if (Count == 0)
        {
            return;
        }
        var coefficients = Cast<TData>().Coefficients;
        var hBuffer = coefficients.Data.Reverse().Concat(coefficients.Data.Reverse()).ToArray();
        var xBuffer = new TData[Count];
        for (var i = 0; i < signal.Length; i++)
        {
            var bufIndex = i % Count;
            xBuffer[bufIndex] = signal[i];
            signal[i] = default;
            var hIndex = Count - 1 - bufIndex;
            for (var j = 0; j < Count; j++)
            {
                signal[i] += hBuffer[hIndex + j] * xBuffer[j];
            }
        }
    }
}
