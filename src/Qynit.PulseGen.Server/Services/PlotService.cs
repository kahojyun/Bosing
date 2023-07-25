using Microsoft.AspNetCore.SignalR;

using Qynit.PulseGen.Server.Hubs;

namespace Qynit.PulseGen.Server.Services;

public class PlotService : IPlotService
{
    private readonly ILogger<PlotService> _logger;
    private readonly IHubContext<PlotHub, IPlotClient> _hubContext;
    private readonly Dictionary<string, ArcUnsafe<PooledComplexArray<float>>> _waveforms = new();

    public PlotService(ILogger<PlotService> logger, IHubContext<PlotHub, IPlotClient> hubContext)
    {
        _logger = logger;
        _hubContext = hubContext;
    }

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

    public bool TryGetPlot(string name, out ArcUnsafe<PooledComplexArray<float>> waveform)
    {
        lock (_waveforms)
        {
            if (_waveforms.TryGetValue(name, out var arc))
            {
                waveform = arc.Clone();
                return true;
            }
            waveform = default;
            return false;
        }
    }

    public void UpdatePlots(IReadOnlyDictionary<string, ArcUnsafe<PooledComplexArray<float>>> waveforms)
    {
        lock (_waveforms)
        {
            foreach (var (name, newArc) in waveforms)
            {
                if (_waveforms.TryGetValue(name, out var oldArc))
                {
                    oldArc.Dispose();
                }
                _waveforms[name] = newArc;
            }
        }
        _hubContext.Clients.All.ReceiveNames(waveforms.Keys);
    }
}
