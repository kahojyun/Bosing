namespace Qynit.PulseGen.Server.Services;

public class PlotUpdateEventArgs : EventArgs
{
    public IEnumerable<string> TraceNames { get; }

    public PlotUpdateEventArgs(IEnumerable<string> traceNames)
    {
        TraceNames = traceNames;
    }
}
