namespace Qynit.PulseGen.Server.Services;

public class PlotUpdateEventArgs(IEnumerable<string> traceNames) : EventArgs
{
    public IEnumerable<string> UpdatedSeries { get; } = traceNames;
}
