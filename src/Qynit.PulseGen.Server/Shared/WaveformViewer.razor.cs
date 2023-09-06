using System.Buffers;
using System.Diagnostics;
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
    private JsViewer? _jsViewer;

    protected override void OnInitialized()
    {
        PlotService.PlotUpdate += OnPlotUpdate;
    }

    private void OnPlotUpdate(object? sender, PlotUpdateEventArgs e)
    {
        _ = InvokeAsync(() => _jsViewer?.UpdateSeriesData(e.UpdatedSeries));
    }

    protected override async Task OnAfterRenderAsync(bool firstRender)
    {
        if (firstRender)
        {
            Debug.Assert(_jsViewer is null);
            Debug.Assert(_chart is not null);
            _jsViewer = await JsViewer.CreateAsync(PlotService, JS, _chart.Value);
        }
        if (_jsViewer is not null)
        {
            await _jsViewer.SetAllSeriesAsync(Names ?? Enumerable.Empty<string>());
        }
    }

    public async ValueTask DisposeAsync()
    {
        PlotService.PlotUpdate -= OnPlotUpdate;
        if (_jsViewer is not null)
        {
            await _jsViewer.DisposeAsync();
        }
    }

    private class JsViewer : IAsyncDisposable
    {
        private IPlotService PlotService { get; }
        private IJSObjectReference ObjectReference { get; }
        private Task? UpdateTask { get; set; }
        private CancellationTokenSource Cts { get; } = new();
        private Queue<string> UpdateQueue { get; } = new();
        private List<string> CurrentSeries { get; set; } = new();

        private JsViewer(IPlotService plotService, IJSObjectReference objectReference)
        {
            PlotService = plotService;
            ObjectReference = objectReference;
        }

        public static async Task<JsViewer> CreateAsync(IPlotService plotService, IJSRuntime js, ElementReference element)
        {
            await using var module = await js.ImportComponentModule<WaveformViewer>();
            var objectReference = await module.InvokeAsync<IJSObjectReference>("Viewer.create", element);
            return new JsViewer(plotService, objectReference);
        }

        public async ValueTask SetAllSeriesAsync(IEnumerable<string> allSeries)
        {
            if (CurrentSeries.SequenceEqual(allSeries))
            {
                return;
            }
            await ObjectReference.InvokeVoidAsync("setAllSeries", allSeries);
            var newSeries = allSeries.Except(CurrentSeries);
            UpdateSeriesData(newSeries);
            CurrentSeries = allSeries.ToList();
        }

        public void UpdateSeriesData(IEnumerable<string> updatedSeries)
        {
            foreach (var series in updatedSeries.Except(UpdateQueue))
            {
                UpdateQueue.Enqueue(series);
            }
            if (UpdateTask is null || UpdateTask.IsCompleted)
            {
                UpdateTask = UpdateInBackground(Cts.Token);
            }
        }

        private async Task UpdateInBackground(CancellationToken token)
        {
            while (!token.IsCancellationRequested && UpdateQueue.TryDequeue(out var name))
            {
                await SetSeriesDataAsync(name);
            }
        }

        private async ValueTask SetSeriesDataAsync(string name)
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
                    await ObjectReference.InvokeVoidAsync("setSeriesData", name, dataType, isReal, streamRef);
                }
            }
        }

        public async ValueTask DisposeAsync()
        {
            Cts.Cancel();
            Cts.Dispose();
            try
            {
                await ObjectReference.InvokeVoidAsync("dispose");
                await ObjectReference.DisposeAsync();
            }
            catch (JSDisconnectedException)
            {
            }
        }
    }
}
