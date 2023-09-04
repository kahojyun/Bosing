namespace Qynit.PulseGen.Server.Services;

public interface IPlotService
{
    void UpdatePlots(IReadOnlyDictionary<string, ArcUnsafe<PooledComplexArray<float>>> waveforms);
    bool TryGetPlot(string name, out ArcUnsafe<PooledComplexArray<float>> waveform);
    IEnumerable<string> GetNames();
    void ClearPlots();
    event EventHandler<PlotUpdateEventArgs>? PlotUpdate;
}
