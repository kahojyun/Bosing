using System.Collections;
using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Bosing;
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
        var bufferLength = Vector.IsHardwareAccelerated ? MCeil(Count, Vector<TData>.Count) : Count;
        var coefficients = Cast<TData>().Coefficients.Data;
        var hBuffer = new TData[bufferLength * 2];
        for (var i = 0; i < coefficients.Length; i++)
        {
            hBuffer[bufferLength - 1 - i] = coefficients[i];
            hBuffer[2 * bufferLength - 1 - i] = coefficients[i];
        }
        var xBuffer = new TData[bufferLength];
        for (var i = 0; i < signal.Length; i++)
        {
            var xIndex = i % bufferLength;
            xBuffer[xIndex] = signal[i];
            var hIndex = bufferLength - 1 - xIndex;
            Debug.Assert(hIndex >= 0);
            Debug.Assert(hIndex + bufferLength <= hBuffer.Length);
            ref var x = ref MemoryMarshal.GetArrayDataReference(xBuffer);
            ref var h = ref MemoryMarshal.GetArrayDataReference(hBuffer);
            if (Vector.IsHardwareAccelerated)
            {
                Debug.Assert(bufferLength % Vector<TData>.Count == 0);
                var acc = Vector<TData>.Zero;
                for (var j = 0; j < bufferLength; j += Vector<TData>.Count)
                {
                    var xv = Unsafe.As<TData, Vector<TData>>(ref Unsafe.Add(ref x, j));
                    var hv = Unsafe.As<TData, Vector<TData>>(ref Unsafe.Add(ref h, hIndex + j));
                    acc += xv * hv;
                }
                signal[i] = Vector.Sum(acc);
            }
            else
            {
                signal[i] = default;
                for (var j = 0; j < bufferLength; j++)
                {
                    signal[i] += Unsafe.Add(ref x, j) * Unsafe.Add(ref h, hIndex + j);
                }
            }

        }
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static int MCeil(int value, int multiple)
    {
        return (value + multiple - 1) / multiple * multiple;
    }
}
