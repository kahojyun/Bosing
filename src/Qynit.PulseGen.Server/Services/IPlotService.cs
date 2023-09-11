namespace Qynit.PulseGen.Server.Services;

public interface IPlotService
{
    void UpdatePlots(IReadOnlyDictionary<string, PlotData> waveforms);
    bool TryGetPlot(string name, out PlotData waveform);
    IEnumerable<string> GetNames();
    void ClearPlots();
    event EventHandler<PlotUpdateEventArgs>? PlotUpdate;
}
