using MessagePack;
using MessagePack.Formatters;
using MessagePack.Resolvers;

using Microsoft.Fast.Components.FluentUI;

using Qynit.PulseGen;
using Qynit.PulseGen.Server;
using Qynit.PulseGen.Server.Hubs;
using Qynit.PulseGen.Server.Models;
using Qynit.PulseGen.Server.Services;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddRazorPages();
builder.Services.AddServerSideBlazor();
builder.Services.AddFluentUIComponents();
builder.Services.AddSingleton<IPlotService, PlotService>();
var app = builder.Build();

app.UseStaticFiles();
app.UseRouting();

app.MapBlazorHub();
app.MapHub<PlotHub>(PlotHub.Uri);
app.MapFallbackToPage("/_Host");

var resolver = CompositeResolver.Create(
    new IMessagePackFormatter[] { new ComplexArrayFormatter() },
    new[] { StandardResolver.Instance });
var options = MessagePackSerializerOptions.Standard.WithResolver(resolver);

const string contentType = "application/msgpack";

app.MapPost("/api/schedule", async (HttpRequest request, HttpResponse response, CancellationToken token, IPlotService plotService) =>
{
    if (request.ContentType != contentType)
    {
        return Results.BadRequest();
    }

    var pgRequest = await MessagePackSerializer.DeserializeAsync<ScheduleRequest>(request.Body, options, token);
    var runner = new ScheduleRunner(pgRequest);
    var waveforms = runner.Run();
    var floatWaveforms = waveforms.Select(x =>
    {
        var floatArray = new PooledComplexArray<float>(x.Length, false);
        WaveformUtils.ConvertDoubleToFloat(floatArray, x);
        return floatArray;
    }).ToList();
    waveforms.ForEach(x => x.Dispose());
    var arcWaveforms = floatWaveforms.Select(ArcUnsafe.Wrap).ToList();
    plotService.UpdatePlots(pgRequest.ChannelTable!.Zip(arcWaveforms).ToDictionary(x => x.First.Name, x => x.Second.Clone()));
    var pgResponse = new PulseGenResponse(arcWaveforms);
    response.RegisterForDispose(pgResponse);
    return Results.Extensions.MessagePack(pgResponse, options);
})
.WithName("Schedule")
.Accepts<ScheduleRequest>(contentType)
.Produces(StatusCodes.Status400BadRequest);

app.Run();
