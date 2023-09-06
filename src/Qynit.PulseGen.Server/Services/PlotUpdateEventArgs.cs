namespace Qynit.PulseGen.Server.Services;

public class PlotUpdateEventArgs : EventArgs
{
    public IEnumerable<string> UpdatedSeries { get; }

    public PlotUpdateEventArgs(IEnumerable<string> traceNames)
    {
        UpdatedSeries = traceNames;
    }
}
