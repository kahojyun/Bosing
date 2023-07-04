namespace Qynit.PulseGen;

public record Channel(string Name)
{
    public static implicit operator Channel(string name)
    {
        return new(name);
    }
}
