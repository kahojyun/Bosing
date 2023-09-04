using System.Buffers;
using System.Collections.Concurrent;
using System.IO.Pipelines;
using System.Runtime.InteropServices;
using System.Threading.Channels;

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
    private DotNetObjectReference<WaveformViewer>? _objRef;
    private Task? _renderTask;
    private CancellationTokenSource? _renderCts;
    private readonly Channel<string> _renderQueue = Channel.CreateUnbounded<string>();
    private readonly ConcurrentDictionary<string, bool> _channelNeedUpdate = new();
    protected override async Task OnAfterRenderAsync(bool firstRender)
    {
        if (firstRender)
        {
            await using var module = await JS.ImportComponentModule<WaveformViewer>();
            _objRef = DotNetObjectReference.Create(this);
            _viewer = await module.InvokeAsync<IJSObjectReference>("Viewer.create", _chart, _objRef);
            _renderCts = new();
            _renderTask = RenderInBackground(_renderCts.Token);
        }

        if (_viewer is not null)
        {
            await SetAllSeries(Names ?? Enumerable.Empty<string>());
            foreach (var name in Names ?? Enumerable.Empty<string>())
            {
                _channelNeedUpdate[name] = true;
                await _renderQueue.Writer.WriteAsync(name);
            }
        }
    }

    private async Task RenderInBackground(CancellationToken token)
    {
        while (!token.IsCancellationRequested)
        {
            var name = await _renderQueue.Reader.ReadAsync(token);
            if (_channelNeedUpdate.TryUpdate(name, false, true))
            {
                await SetSeriesData(name, token);
            }
        }
    }

    private async ValueTask SetAllSeries(IEnumerable<string> names)
    {
        await _viewer!.InvokeVoidAsync("setAllSeries", names);
    }

    private async ValueTask SetSeriesData(string name, CancellationToken token)
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
                await _viewer!.InvokeVoidAsync("setSeriesData", token, name, dataType, isReal, streamRef);
            }
        }
    }

    public async ValueTask DisposeAsync()
    {
        _renderCts?.Cancel();
        if (_renderTask is not null)
        {
            try
            {
                await _renderTask;
            }
            catch (OperationCanceledException)
            {
            }
        }

        _renderCts?.Dispose();
        _objRef?.Dispose();
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