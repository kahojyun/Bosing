using System.Numerics;
using System.Runtime.CompilerServices;

namespace Qynit.Pulsewave;
internal record PulseList<T>
    where T : unmanaged, INumber<T>, ITrigonometricFunctions<T>
{
    public double TimeOffset { get; init; }
    public IqPair<T> AmplitudeMultiplier { get; init; } = IqPair<T>.One;

    internal IReadOnlyDictionary<BinInfo, IReadOnlyList<BinItem>> Items { get; }

    private PulseList(IReadOnlyDictionary<BinInfo, IReadOnlyList<BinItem>> items)
    {
        Items = items;
    }

    public static PulseList<T> operator +(PulseList<T> left, PulseList<T> right)
    {
        var newItems = new Dictionary<BinInfo, IReadOnlyList<BinItem>>(left.Items);
        foreach (var (rightKey, rightList) in right.Items)
        {
            if (newItems.TryGetValue(rightKey, out var leftList))
            {
                var newList = MergeListWithInfo(leftList, rightList, left, right);
                newItems[rightKey] = newList;
            }
            else
            {
                newItems.Add(rightKey, rightList);
            }
        }
        return new PulseList<T>(newItems);
    }

    public static PulseList<T> operator *(PulseList<T> left, IqPair<T> right)
    {
        return left with { AmplitudeMultiplier = left.AmplitudeMultiplier * right };
    }
    public static PulseList<T> operator *(IqPair<T> left, PulseList<T> right)
    {
        return right with { AmplitudeMultiplier = right.AmplitudeMultiplier * left };
    }
    public PulseList<T> TimeShifted(double timeOffset)
    {
        return this with { TimeOffset = TimeOffset + timeOffset };
    }

    private static IReadOnlyList<BinItem> MergeListWithInfo(IReadOnlyList<BinItem> list1, IReadOnlyList<BinItem> list2, PulseList<T> info1, PulseList<T> info2)
    {
        var newList = new List<BinItem>(list1.Count + list2.Count);
        var i = 0;
        var j = 0;
        var timeOffset1 = info1.TimeOffset;
        var timeOffset2 = info2.TimeOffset;
        var multiplier1 = info1.AmplitudeMultiplier;
        var multiplier2 = info2.AmplitudeMultiplier;
        while (i < list1.Count && j < list2.Count)
        {
            var item1 = list1[i];
            var item2 = list2[j];
            var newTime1 = timeOffset1 + item1.Time;
            var newTime2 = timeOffset2 + item2.Time;
            if (newTime1 < newTime2)
            {
                var newItem = new BinItem(newTime1, item1.Amplitude * multiplier1);
                newList.Add(newItem);
                i++;
            }
            else if (newTime1 > newTime2)
            {
                var newItem = new BinItem(newTime2, item2.Amplitude * multiplier2);
                newList.Add(newItem);
                j++;
            }
            else
            {
                var newItem = new BinItem(newTime1, item1.Amplitude * multiplier1 + item2.Amplitude * multiplier2);
                newList.Add(newItem);
                i++;
                j++;
            }
        }
        while (i < list1.Count)
        {
            var item1 = list1[i];
            var newTime1 = timeOffset1 + item1.Time;
            var newItem = new BinItem(newTime1, item1.Amplitude * multiplier1);
            newList.Add(newItem);
            i++;
        }
        while (j < list2.Count)
        {
            var item2 = list2[j];
            var newTime2 = timeOffset2 + item2.Time;
            var newItem = new BinItem(newTime2, item2.Amplitude * multiplier2);
            newList.Add(newItem);
            j++;
        }
        return newList;
    }

    internal record BinInfo(Envelope Envelope, double Frequency);
    internal readonly record struct PulseAmplitude(IqPair<T> Amplitude, IqPair<T> DragAmplitude)
    {
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static PulseAmplitude operator +(PulseAmplitude left, PulseAmplitude right)
        {
            return new PulseAmplitude(left.Amplitude + right.Amplitude, left.DragAmplitude + right.DragAmplitude);
        }
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static PulseAmplitude operator *(PulseAmplitude left, IqPair<T> right)
        {
            return new PulseAmplitude(left.Amplitude * right, left.DragAmplitude * right);
        }
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static PulseAmplitude operator *(IqPair<T> left, PulseAmplitude right)
        {
            return new PulseAmplitude(left * right.Amplitude, left * right.DragAmplitude);
        }
    }
    internal record BinItem(double Time, PulseAmplitude Amplitude);
    internal class Builder
    {
        private readonly Dictionary<BinInfo, List<BinItem>> _items = new();
        public void Add(Envelope envelope, double frequency, double time, T amplitude, T phase, T dragCoefficient)
        {
            var cAmplitude = IqPair<T>.FromPolarCoordinates(amplitude, phase);
            var cDragAmplitude = cAmplitude * IqPair<T>.ImaginaryOne * dragCoefficient;
            Add(envelope, frequency, time, cAmplitude, cDragAmplitude);
        }
        public void Add(Envelope envelope, double frequency, double time, IqPair<T> amplitude, IqPair<T> dragAmplitude)
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
        public PulseList<T> Build()
        {
            foreach (var item in _items.Values)
            {
                SortAndCompress(item);
            }
            var result = new PulseList<T>(_items.ToDictionary(x => x.Key, x => (IReadOnlyList<BinItem>)x.Value));
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
