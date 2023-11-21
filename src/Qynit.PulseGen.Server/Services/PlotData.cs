namespace Qynit.PulseGen.Server.Services;

public readonly struct PlotData(string name, ArcUnsafe<PooledComplexArray<float>> waveform, double dt) : IDisposable
{
    public string Name { get; } = name;
    public ArcUnsafe<PooledComplexArray<float>> Waveform { get; } = waveform;
    public double Dt { get; } = dt;

    public PlotData Clone()
    {
        return new(Name, Waveform.Clone(), Dt);
    }

    public void Dispose()
    {
        Waveform.Dispose();
    }
}
