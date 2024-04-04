using System.Numerics;
using System.Runtime.CompilerServices;

namespace Bosing;
public record PulseList
{
    public static readonly PulseList Empty = new();
    public double TimeOffset { get; init; }
    public Complex AmplitudeMultiplier { get; init; } = Complex.One;
    public SignalFilter<double> Filter { get; init; } = SignalFilter<double>.Empty;

    internal IReadOnlyDictionary<BinInfo, IReadOnlyList<BinItem>> Items { get; }

    private PulseList()
    {
        Items = new Dictionary<BinInfo, IReadOnlyList<BinItem>>();
    }

    private PulseList(IReadOnlyDictionary<BinInfo, IReadOnlyList<BinItem>> items)
    {
        Items = items;
    }

    public static PulseList operator *(PulseList left, Complex right)
    {
        return left with { AmplitudeMultiplier = left.AmplitudeMultiplier * right };
    }
    public static PulseList operator *(Complex left, PulseList right)
    {
        return right with { AmplitudeMultiplier = right.AmplitudeMultiplier * left };
    }
    public PulseList TimeShifted(double timeOffset)
    {
        return this with { TimeOffset = TimeOffset + timeOffset };
    }
    public PulseList Filtered(SignalFilter<double> filter)
    {
        return this with { Filter = SignalFilter<double>.Concat(Filter, filter) };
    }

    public static PulseList Sum(double timeTolerance, double ampTolerance, params PulseList[] pulseLists)
    {
        return Sum(pulseLists, timeTolerance, ampTolerance);
    }

    public static PulseList Sum(IEnumerable<PulseList> pulseLists, double timeTolerance, double ampTolerance)
    {
        var newItems = new Dictionary<BinInfo, IReadOnlyList<BinItem>>();
        foreach (var pulseList in pulseLists)
        {
            foreach (var (key, list) in pulseList.Items)
            {
                var newKey = key with
                {
                    Delay = key.Delay + pulseList.TimeOffset,
                    Filter = SignalFilter<double>.Concat(key.Filter, pulseList.Filter),
                };
                var newList = newItems.TryGetValue(newKey, out var oldList)
                    ? AddMultiply(oldList, list, pulseList.AmplitudeMultiplier, timeTolerance)
                    : ApplyMultiplier(list, pulseList.AmplitudeMultiplier, ampTolerance);
                newItems[newKey] = newList;
            }
        }
        return new PulseList(newItems);
    }

    private static IReadOnlyList<BinItem> AddMultiply(IReadOnlyList<BinItem> list, IReadOnlyList<BinItem> other, Complex multiplier, double timeTolerance)
    {
        if (multiplier == Complex.Zero || other.Count == 0)
        {
            return list;
        }
        var newList = new List<BinItem>(list.Count);
        var i = 0;
        var j = 0;
        while (i < list.Count && j < other.Count)
        {
            var item1 = list[i];
            var item2 = other[j];
            if (item1.Time + timeTolerance < item2.Time)
            {
                newList.Add(item1);
                i++;
            }
            else if (item1.Time > item2.Time + timeTolerance)
            {
                var newItem = item2 with { Amplitude = item2.Amplitude * multiplier };
                newList.Add(newItem);
                j++;
            }
            else
            {
                var newItem = new BinItem(item1.Time, item1.Amplitude + item2.Amplitude * multiplier);
                newList.Add(newItem);
                i++;
                j++;
            }
        }
        while (i < list.Count)
        {
            newList.Add(list[i]);
            i++;
        }
        while (j < other.Count)
        {
            var item2 = other[j];
            var newItem = item2 with { Amplitude = item2.Amplitude * multiplier };
            newList.Add(newItem);
            j++;
        }
        return newList;
    }

    private static IReadOnlyList<BinItem> ApplyMultiplier(IReadOnlyList<BinItem> list, Complex multiplier, double ampTolerance)
    {
        if (MathUtils.IsApproximatelyZero(multiplier.Imaginary, ampTolerance))
        {
            if (MathUtils.IsApproximatelyZero(multiplier.Real, ampTolerance))
            {
                return Array.Empty<BinItem>();
            }
            if (MathUtils.IsApproximatelyEqual(multiplier.Real, 1, ampTolerance))
            {
                return list;
            }
        }
        return list.Select(item => new BinItem(item.Time, item.Amplitude * multiplier)).ToArray();
    }

    internal readonly record struct BinInfo(Envelope Envelope, double GlobalFrequency, double LocalFrequency, double Delay)
    {
        public SignalFilter<double> Filter { get; init; } = SignalFilter<double>.Empty;
    }
    internal readonly record struct PulseAmplitude(Complex Amplitude, Complex DragAmplitude)
    {
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static PulseAmplitude operator +(PulseAmplitude left, PulseAmplitude right)
        {
            return new PulseAmplitude(left.Amplitude + right.Amplitude, left.DragAmplitude + right.DragAmplitude);
        }
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static PulseAmplitude operator *(PulseAmplitude left, Complex right)
        {
            return new PulseAmplitude(left.Amplitude * right, left.DragAmplitude * right);
        }
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static PulseAmplitude operator *(Complex left, PulseAmplitude right)
        {
            return new PulseAmplitude(left * right.Amplitude, left * right.DragAmplitude);
        }
    }
    internal readonly record struct BinItem(double Time, PulseAmplitude Amplitude);
    internal class Builder(double timeTolerance)
    {
        public double TimeTolerance { get; init; } = timeTolerance;

        private readonly Dictionary<BinInfo, List<BinItem>> _items = [];
        public void Add(Envelope envelope, double globalFrequency, double localFrequency, double time, double amplitude, double phase, double dragCoefficient)
        {
            if (amplitude == 0)
            {
                return;
            }
            var cAmplitude = Complex.FromPolarCoordinates(amplitude, Math.Tau * phase);
            var cDragAmplitude = cAmplitude * Complex.ImaginaryOne * dragCoefficient;
            Add(envelope, globalFrequency, localFrequency, 0, time, cAmplitude, cDragAmplitude);
        }
        public void Add(Envelope envelope, double globalFrequency, double localFrequency, double delay, double time, Complex amplitude, Complex dragAmplitude)
        {
            var binInfo = new BinInfo(envelope, globalFrequency, localFrequency, delay);
            var item = new BinItem(time, new PulseAmplitude(amplitude, dragAmplitude));
            Add(binInfo, item);
        }
        public void Add(BinInfo binInfo, BinItem item)
        {
            if (!_items.TryGetValue(binInfo, out var list))
            {
                list = [];
                _items.Add(binInfo, list);
            }
            list.Add(item);
        }
        public PulseList Build()
        {
            foreach (var item in _items.Values)
            {
                SortAndCompress(item, TimeTolerance);
            }
            var result = new PulseList(_items.ToDictionary(x => x.Key, x => (IReadOnlyList<BinItem>)x.Value));
            _items.Clear();
            return result;
        }
        private static void SortAndCompress(List<BinItem> item, double tolerance)
        {
            item.Sort((a, b) => a.Time.CompareTo(b.Time));
            var i = 0;
            var j = 1;
            while (j < item.Count)
            {
                var item1 = item[i];
                var item2 = item[j];
                if (MathUtils.IsApproximatelyEqual(item1.Time, item2.Time, tolerance))
                {
                    item[i] = new BinItem(item1.Time, item1.Amplitude + item2.Amplitude);
                    j++;
                }
                else
                {
                    i++;
                    if (i != j)
                    {
                        item[i] = item2;
                    }
                    j++;
                }
            }
            item.RemoveRange(i + 1, item.Count - i - 1);
        }
    }
}
