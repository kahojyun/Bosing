using Microsoft.AspNetCore.Components;
using Microsoft.AspNetCore.SignalR.Client;
using Microsoft.JSInterop;

using Qynit.PulseGen.Server.Hubs;
using Qynit.PulseGen.Server.Services;

namespace Qynit.PulseGen.Server.Pages;

public sealed partial class Index : IAsyncDisposable, IPlotClient
{
    [Inject]
    private NavigationManager Navigation { get; set; } = default!;

    [Inject]
    private IJSRuntime JS { get; set; } = default!;

    [Inject]
    private IPlotService PlotService { get; set; } = default!;

    private HubConnection? _hubConnection;
    private string _nameFilter = string.Empty;
    private class Trace
    {
        public string Name { get; set; } = string.Empty;
        public bool Visible { get; set; }
        public bool NeedUpdate { get; set; }
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
        Traces = names.Select(x => new Trace { Name = x, Visible = true, NeedUpdate = true }).ToList();
    }

    protected override async Task OnInitializedAsync()
    {
        _hubConnection = new HubConnectionBuilder().WithUrl(Navigation.ToAbsoluteUri(PlotHub.Uri)).Build();
        _hubConnection.On<IEnumerable<string>>(nameof(ReceiveNames), ReceiveNames);
        await _hubConnection.StartAsync();
    }

    public async Task ReceiveNames(IEnumerable<string> names)
    {
        var tracesLookUp = Traces.ToDictionary(x => x.Name);
        foreach (var name in names)
        {
            if (tracesLookUp.TryGetValue(name, out var trace))
            {
                trace.NeedUpdate = true;
            }
            else
            {
                Traces.Add(new Trace { Name = name, Visible = true, NeedUpdate = true });
            }
        }

        await InvokeAsync(StateHasChanged);
    }

    public async ValueTask DisposeAsync()
    {
        if (_hubConnection is not null)
        {
            await _hubConnection.DisposeAsync();
        }
    }
}
