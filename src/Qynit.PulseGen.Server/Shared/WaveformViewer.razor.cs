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
    private ElementReference? _overview;
    private JsViewer? _jsViewer;

    protected override void OnInitialized()
    {
        PlotService.PlotUpdate += OnPlotUpdate;
    }

    private void OnPlotUpdate(object? sender, PlotUpdateEventArgs e)
    {
        _ = InvokeAsync(() => _jsViewer?.UpdateExistingSeries(e.UpdatedSeries));
    }

    protected override async Task OnAfterRenderAsync(bool firstRender)
    {
        if (firstRender)
        {
            Debug.Assert(_jsViewer is null);
            Debug.Assert(_chart is not null);
            Debug.Assert(_overview is not null);
            _jsViewer = await JsViewer.CreateAsync(PlotService, JS, _chart.Value, _overview.Value);
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
        private List<string> CurrentSeries { get; set; } = [];

        private JsViewer(IPlotService plotService, IJSObjectReference objectReference)
        {
            PlotService = plotService;
            ObjectReference = objectReference;
        }

        public static async Task<JsViewer> CreateAsync(IPlotService plotService, IJSRuntime js, ElementReference chartElement, ElementReference overviewElement)
        {
            await using var module = await js.ImportComponentModule<WaveformViewer>();
            var objectReference = await module.InvokeAsync<IJSObjectReference>("Viewer.create", chartElement, overviewElement);
            return new JsViewer(plotService, objectReference);
        }

        public async ValueTask SetAllSeriesAsync(IEnumerable<string> allSeries)
        {
            var allSeriesList = allSeries.ToList();
            if (CurrentSeries.SequenceEqual(allSeriesList))
            {
                return;
            }
            await JsSetAllSeries(allSeriesList);
            var newSeries = allSeriesList.Except(CurrentSeries);
            CurrentSeries = allSeriesList;
            EnqueueUpdates(newSeries);
        }

        public void UpdateExistingSeries(IEnumerable<string> updatedSeries)
        {
            EnqueueUpdates(updatedSeries.Intersect(CurrentSeries));
        }

        private void EnqueueUpdates(IEnumerable<string> updatedSeries)
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
                if (!CurrentSeries.Contains(name))
                {
                    continue;
                }
                await SetSeriesDataAsync(name);
            }
        }

        private async ValueTask SetSeriesDataAsync(string name)
        {
            if (PlotService.TryGetPlot(name, out var plotData))
            {
                using (plotData)
                {
                    var pooledArray = plotData.Waveform.Target;
                    var dt = plotData.Dt;
                    var pipe = new Pipe();
                    var writer = pipe.Writer;
                    writer.Write(MemoryMarshal.AsBytes(pooledArray.DataI));
                    var isReal = pooledArray.IsReal;
                    if (!isReal)
                    {
                        writer.Write(MemoryMarshal.AsBytes(pooledArray.DataQ));
                    }
                    writer.Complete();
                    using var streamRef = new DotNetStreamReference(pipe.Reader.AsStream());
                    var dataType = DataType.Float32;
                    await JsSetSeriesData(name, dataType, isReal, dt, streamRef);
                }
            }
        }

        private ValueTask JsSetAllSeries(IEnumerable<string> allSeries)
        {
            return ObjectReference.InvokeVoidAsync("setAllSeries", allSeries);
        }

        private ValueTask JsSetSeriesData(string name, DataType dataType, bool isReal, double dt, DotNetStreamReference streamRef)
        {
            return ObjectReference.InvokeVoidAsync("setSeriesData", name, dataType, isReal, dt, streamRef);
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
