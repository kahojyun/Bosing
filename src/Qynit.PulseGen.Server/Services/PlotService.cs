namespace Qynit.PulseGen.Server.Services;

public class PlotService : IPlotService
{
    private readonly Dictionary<string, PlotData> _waveforms = new();

    public event EventHandler<PlotUpdateEventArgs>? PlotUpdate;

    public void ClearPlots()
    {
        lock (_waveforms)
        {
            foreach (var arc in _waveforms.Values)
            {
                arc.Dispose();
            }
            _waveforms.Clear();
        }
    }

    public IEnumerable<string> GetNames()
    {
        lock (_waveforms)
        {
            return _waveforms.Keys.ToArray();
        }
    }

    public bool TryGetPlot(string name, out PlotData waveform)
    {
        lock (_waveforms)
        {
            if (_waveforms.TryGetValue(name, out var plotData))
            {
                waveform = plotData.Clone();
                return true;
            }
            waveform = default;
            return false;
        }
    }

    public void UpdatePlots(IReadOnlyDictionary<string, PlotData> waveforms)
    {
        lock (_waveforms)
        {
            foreach (var (name, newData) in waveforms)
            {
                if (_waveforms.TryGetValue(name, out var oldData))
                {
                    oldData.Dispose();
                }
                _waveforms[name] = newData;
            }
            PlotUpdate?.Invoke(this, new(_waveforms.Keys));
        }
    }
}
