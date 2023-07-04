using System.Numerics;
using System.Runtime.CompilerServices;

namespace Qynit.Pulsewave;
internal record PulseList
{
    public static readonly PulseList Empty = new();
    public double TimeOffset { get; init; }
    public Complex AmplitudeMultiplier { get; init; } = Complex.One;

    internal IReadOnlyDictionary<BinInfo, IReadOnlyList<BinItem>> Items { get; }

    private PulseList()
    {
        Items = new Dictionary<BinInfo, IReadOnlyList<BinItem>>();
    }

    private PulseList(IReadOnlyDictionary<BinInfo, IReadOnlyList<BinItem>> items)
    {
        Items = items;
    }

    public static PulseList operator +(PulseList left, PulseList right)
    {
        return Sum(left, right);
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

    public static PulseList Sum(params PulseList[] pulseLists)
    {
        return Sum((IEnumerable<PulseList>)pulseLists);
    }

    public static PulseList Sum(IEnumerable<PulseList> pulseLists)
    {
        var newItems = new Dictionary<BinInfo, IReadOnlyList<BinItem>>();
        foreach (var pulseList in pulseLists)
        {
            foreach (var (key, list) in pulseList.Items)
            {
                var newList = newItems.TryGetValue(key, out var oldList)
                    ? AddApplyInfo(oldList, list, pulseList.TimeOffset, pulseList.AmplitudeMultiplier)
                    : ApplyInfo(list, pulseList.TimeOffset, pulseList.AmplitudeMultiplier);
                newItems[key] = newList;
            }
        }
        return new PulseList(newItems);
    }

    private static IReadOnlyList<BinItem> AddApplyInfo(IReadOnlyList<BinItem> list, IReadOnlyList<BinItem> other, double timeOffset, Complex multiplier)
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
            var newTime1 = item1.Time;
            var newTime2 = timeOffset + item2.Time;
            if (newTime1 < newTime2)
            {
                newList.Add(item1);
                i++;
            }
            else if (newTime1 > newTime2)
            {
                var newItem = new BinItem(newTime2, item2.Amplitude * multiplier);
                newList.Add(newItem);
                j++;
            }
            else
            {
                var newItem = new BinItem(newTime1, item1.Amplitude + item2.Amplitude * multiplier);
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
            var newTime2 = timeOffset + item2.Time;
            var newItem = new BinItem(newTime2, item2.Amplitude * multiplier);
            newList.Add(newItem);
            j++;
        }
        return newList;
    }

    private static IReadOnlyList<BinItem> ApplyInfo(IReadOnlyList<BinItem> list, double timeOffset, Complex multiplier)
    {
        return multiplier == Complex.Zero
            ? Array.Empty<BinItem>()
            : (IReadOnlyList<PulseList.BinItem>)list.Select(item => new BinItem(timeOffset + item.Time, item.Amplitude * multiplier)).ToArray();
    }

    internal readonly record struct BinInfo(Envelope Envelope, double Frequency);
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
    internal class Builder
    {
        private readonly Dictionary<BinInfo, List<BinItem>> _items = new();
        public void Add(Envelope envelope, double frequency, double time, double amplitude, double phase, double dragCoefficient)
        {
            var cAmplitude = Complex.FromPolarCoordinates(amplitude, phase);
            var cDragAmplitude = cAmplitude * Complex.ImaginaryOne * dragCoefficient;
            Add(envelope, frequency, time, cAmplitude, cDragAmplitude);
        }
        public void Add(Envelope envelope, double frequency, double time, Complex amplitude, Complex dragAmplitude)
        {
            var binInfo = new BinInfo(envelope, frequency);
            var item = new BinItem(time, new PulseAmplitude(amplitude, dragAmplitude));
            Add(binInfo, item);
        }
        public void Add(BinInfo binInfo, BinItem item)
        {
            if (!_items.TryGetValue(binInfo, out var list))
            {
                list = new List<BinItem>();
                _items.Add(binInfo, list);
            }
            list.Add(item);
        }
        public PulseList Build()
        {
            foreach (var item in _items.Values)
            {
                SortAndCompress(item);
            }
            var result = new PulseList(_items.ToDictionary(x => x.Key, x => (IReadOnlyList<BinItem>)x.Value));
            _items.Clear();
            return result;
        }
        private static void SortAndCompress(List<BinItem> item)
        {
            item.Sort((a, b) => a.Time.CompareTo(b.Time));
            var i = 0;
            var j = 1;
            while (j < item.Count)
            {
                var item1 = item[i];
                var item2 = item[j];
                if (item1.Time == item2.Time)
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
