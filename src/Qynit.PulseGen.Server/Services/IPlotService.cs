namespace Qynit.PulseGen.Server.Services;

public interface IPlotService
{
    void UpdatePlots(IReadOnlyDictionary<string, ArcUnsafe<PooledComplexArray<double>>> waveforms);
    bool TryGetPlot(string name, out ArcUnsafe<PooledComplexArray<double>> waveform);
    IEnumerable<string> GetNames();
    void ClearPlots();
}
