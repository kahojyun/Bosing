namespace Qynit.PulseGen;

internal readonly record struct ValueArray<T>
{
    public T[] Data { get; init; }
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
}
