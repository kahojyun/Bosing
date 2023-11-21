using System.Reflection;

using MessagePack;
using MessagePack.Resolvers;

using Microsoft.AspNetCore.StaticFiles;
using Microsoft.Fast.Components.FluentUI;

using Qynit.PulseGen.Server.Models;
using Qynit.PulseGen.Server.Services;

namespace Qynit.PulseGen.Server;

public sealed class Server : IDisposable
{
    internal static MessagePackSerializerOptions MessagePackSerializerOptions { get; } =
        MessagePackSerializerOptions.Standard.WithResolver(
            CompositeResolver.Create([new ComplexArrayFormatter()], [StandardResolver.Instance]));

    private readonly WebApplication _app;
    private Server(WebApplication app)
    {
        _app = app;
    }

    public static Server CreateApp(string[] args, bool embedded)
    {
        var builder = embedded ? CreateBuilderForEmbedded(args) : WebApplication.CreateBuilder(args);
        builder.Services.AddRazorPages();
        builder.Services.AddServerSideBlazor();
        builder.Services.AddFluentUIComponents();
        builder.Services.AddSingleton<IPlotService, PlotService>();
        if (embedded)
        {
            builder.Services.AddSingleton<IHostLifetime, NopLifeTime>();
        }
        var app = builder.Build();

        app.UseStaticFiles();
        app.ServeSciChartWasm();

        app.UseRouting();

        app.MapBlazorHub();
        app.MapFallbackToPage("/_Host");

        app.AddScheduleApi();

        return new Server(app);
    }

    private static WebApplicationBuilder CreateBuilderForEmbedded(string[] args)
    {
        var assemblyPath = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)!;
        var webRootPath = Path.Combine(assemblyPath, "wwwroot");
        var env = Environment.GetEnvironmentVariable("ASPNETCORE_ENVIRONMENT") ?? Environments.Production;

        var webApplicationOptions = new WebApplicationOptions
        {
            EnvironmentName = env,
            ApplicationName = "Qynit.PulseGen.Server",
            ContentRootPath = assemblyPath,
            WebRootPath = webRootPath,
            Args = args,
        };

        var builder = WebApplication.CreateBuilder(webApplicationOptions);
        return builder;
    }

    public void Run()
    {
        _app.Run();
    }

    public void Start()
    {
        _app.Start();
    }

    public void Stop()
    {
        _app.StopAsync().GetAwaiter().GetResult();
    }

    public void Dispose()
    {
        ((IDisposable)_app).Dispose();
    }

    public IPlotService? GetPlotService()
    {
        return _app.Services.GetService<IPlotService>();
    }
}


internal class NopLifeTime : IHostLifetime
{
    public Task StopAsync(CancellationToken cancellationToken)
    {
        return Task.CompletedTask;
    }

    public Task WaitForStartAsync(CancellationToken cancellationToken)
    {
        return Task.CompletedTask;
    }
}

internal static class BuilderExtensions
{
    internal static void AddScheduleApi(this WebApplication app)
    {
        const string contentType = "application/msgpack";

        app.MapPost("/api/schedule", async (HttpRequest request, HttpResponse response, CancellationToken token, IPlotService plotService) =>
        {
            if (request.ContentType != contentType)
            {
                return Results.BadRequest();
            }

            var pgRequest = await MessagePackSerializer.DeserializeAsync<ScheduleRequest>(request.Body, Server.MessagePackSerializerOptions, token);
            var runner = new ScheduleRunner(pgRequest);
            var waveforms = runner.Run();
            var arcWaveforms = waveforms.Select(ArcUnsafe.Wrap).ToList();
            plotService.UpdatePlots(pgRequest.ChannelTable!.Zip(arcWaveforms).ToDictionary(x => x.First.Name, x => new PlotData(x.First.Name, x.Second.Clone(), 1.0 / x.First.SampleRate)));
            var pgResponse = new PulseGenResponse(arcWaveforms);
            response.RegisterForDispose(pgResponse);
            return Results.Extensions.MessagePack(pgResponse, Server.MessagePackSerializerOptions);
        })
        .WithName("Schedule")
        .Accepts<ScheduleRequest>(contentType)
        .Produces(StatusCodes.Status400BadRequest);
    }

    internal static void ServeSciChartWasm(this WebApplication app)
    {
        var fileExtensionContentTypeProvider = new FileExtensionContentTypeProvider();
        fileExtensionContentTypeProvider.Mappings[".data"] = "application/octet-stream";
        app.UseStaticFiles(new StaticFileOptions
        {
            ContentTypeProvider = fileExtensionContentTypeProvider
        });
    }
}
