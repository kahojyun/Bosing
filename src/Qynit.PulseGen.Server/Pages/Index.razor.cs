using Microsoft.AspNetCore.Components;

using Qynit.PulseGen.Server.Services;

namespace Qynit.PulseGen.Server.Pages;

public sealed partial class Index : IDisposable
{
    [Inject]
    private IPlotService PlotService { get; set; } = default!;

    private string _nameFilter = string.Empty;
    private class Trace
    {
        public string Name { get; set; } = string.Empty;
        public bool Visible { get; set; }
    }

    private List<Trace> Traces { get; set; } = new();
    private IEnumerable<string>? VisibleTraceNames => Traces.Where(p => p.Visible).Select(p => p.Name).ToList();

    IQueryable<Trace> FilteredTraces => Traces.AsQueryable().Where(p => p.Name.Contains(_nameFilter));

    private bool AnyVisible
    {
        get => FilteredTraces.Any(p => p.Visible);
        set
        {
            foreach (var p in FilteredTraces)
            {
                p.Visible = value;
            }
        }
    }

    private bool IsIndeterminate => FilteredTraces.Any(p => p.Visible) && FilteredTraces.Any(p => !p.Visible);

    protected override void OnInitialized()
    {
        var names = PlotService.GetNames();
        Traces = names.Select(x => new Trace { Name = x, Visible = true }).ToList();
        PlotService.PlotUpdate += OnPlotUpdate;
    }

    private void OnPlotUpdate(object? sender, PlotUpdateEventArgs e)
    {
        _ = InvokeAsync(() =>
        {
            var newNames = e.UpdatedSeries.Except(Traces.Select(p => p.Name));
            var newTraces = newNames.Select(x => new Trace { Name = x, Visible = true });
            Traces.AddRange(newTraces);
            StateHasChanged();
        });
    }

    public void Dispose()
    {
        PlotService.PlotUpdate -= OnPlotUpdate;
    }
}
