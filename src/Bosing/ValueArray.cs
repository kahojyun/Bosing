using System.Collections;

namespace Bosing;

internal readonly record struct ValueArray<T> : IReadOnlyList<T>
{
    public T[] Data { get; init; }

    public int Count => Data.Length;

    public T this[int index] => Data[index];

    public ValueArray(IEnumerable<T> values)
    {
        Data = values.ToArray();
    }
    public bool Equals(ValueArray<T> other)
    {
        return Data.SequenceEqual(other.Data);
    }
    public override int GetHashCode()
    {
        var hash = new HashCode();
        foreach (var item in Data)
        {
            hash.Add(item);
        }
        return hash.ToHashCode();
    }

    public IEnumerator<T> GetEnumerator()
    {
        return ((IReadOnlyList<T>)Data).GetEnumerator();
    }

    IEnumerator IEnumerable.GetEnumerator()
    {
        return Data.GetEnumerator();
    }
}
