namespace Qynit.Pulsewave;

public record Channel(string Name)
{
    public static implicit operator Channel(string name) => new(name);
}
