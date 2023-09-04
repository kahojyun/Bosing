using System.Buffers;
using System.IO.Pipelines;
using System.Runtime.InteropServices;

using Microsoft.AspNetCore.Components;

using Microsoft.JSInterop;

using Qynit.PulseGen.Server.Models;
using Qynit.PulseGen.Server.Services;

namespace Qynit.PulseGen.Server.Shared;

public sealed partial class WaveformViewer : IAsyncDisposable
{
    [Parameter]
    public IEnumerable<string>? Names { get; set; }

    [Parameter, EditorRequired]
    public IPlotService PlotService { get; set; } = default!;

    [Inject]
    private IJSRuntime JS { get; set; } = default!;

    private ElementReference? _chart;
    private IJSObjectReference? _viewer;
    private Task? _renderTask;
    private readonly CancellationTokenSource _renderCts = new();
    // Blazor use a "single-threaded" synchronization context, so it's safe to use a normal queue
    private readonly Queue<string> _renderQueue = new();

    protected override async Task OnAfterRenderAsync(bool firstRender)
    {
        if (firstRender)
        {
            await using var module = await JS.ImportComponentModule<WaveformViewer>();
            _viewer = await module.InvokeAsync<IJSObjectReference>("Viewer.create", _chart);
        }
        if (_viewer is not null && Names is not null)
        {
            await SetAllSeries(Names);
            AddToRenderQueue(Names);
        }
    }

    private void AddToRenderQueue(IEnumerable<string> names)
    {
        foreach (var name in names)
        {
            if (!_renderQueue.Contains(name))
            {
                _renderQueue.Enqueue(name);
            }
        }
        if (_renderTask is null || _renderTask.IsCompleted)
        {
            _renderTask = RenderInBackground(_renderCts.Token);
        }
    }

    private async Task RenderInBackground(CancellationToken token)
    {
        while (!token.IsCancellationRequested && _renderQueue.TryDequeue(out var name))
        {
            await SetSeriesData(name);
        }
    }

    private async ValueTask SetAllSeries(IEnumerable<string> names)
    {
        await _viewer!.InvokeVoidAsync("setAllSeries", names);
    }

    private async ValueTask SetSeriesData(string name)
    {
        if (PlotService.TryGetPlot(name, out var arc))
        {
            using (arc)
            {
                var pipe = new Pipe();
                var writer = pipe.Writer;
                writer.Write(MemoryMarshal.AsBytes(arc.Target.DataI));
                var isReal = arc.Target.IsReal;
                if (!isReal)
                {
                    writer.Write(MemoryMarshal.AsBytes(arc.Target.DataQ));
                }

                writer.Complete();
                using var streamRef = new DotNetStreamReference(pipe.Reader.AsStream());
                var dataType = DataType.Float32;
                await _viewer!.InvokeVoidAsync("setSeriesData", name, dataType, isReal, streamRef);
            }
        }
    }

    public async ValueTask DisposeAsync()
    {
        _renderCts.Cancel();
        _renderCts.Dispose();
        if (_viewer is not null)
        {
            try
            {
                await _viewer.InvokeVoidAsync("dispose");
                await _viewer.DisposeAsync();
            }
            catch (JSDisconnectedException)
            {
            }
        }
    }
}
