namespace Qynit.PulseGen.Server.Services;

public readonly struct PlotData : IDisposable
{
    public string Name { get; }
    public ArcUnsafe<PooledComplexArray<float>> Waveform { get; }
    public double Dt { get; }

    public PlotData(string name, ArcUnsafe<PooledComplexArray<float>> waveform, double dt)
    {
        Name = name;
        Waveform = waveform;
        Dt = dt;
    }

    public PlotData Clone()
    {
        return new(Name, Waveform.Clone(), Dt);
    }

    public void Dispose()
    {
        Waveform.Dispose();
    }
}
