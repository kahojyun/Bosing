using MessagePack;
using MessagePack.Formatters;
using MessagePack.Resolvers;

using Qynit.PulseGen.Server;

var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();

var resolver = CompositeResolver.Create(
    new IMessagePackFormatter[] { new ComplexArrayFormatter() },
    new[] { StandardResolver.Instance });
var options = MessagePackSerializerOptions.Standard.WithResolver(resolver);

const string contentType = "application/msgpack";

app.MapPost("/run", async (HttpRequest request, CancellationToken token) =>
{
    if (request.ContentType != contentType)
    {
        return Results.BadRequest();
    }
    var pgRequest = await MessagePackSerializer.DeserializeAsync<PulseGenRequest>(request.Body, cancellationToken: token);
    var runner = new PulseGenRunner(pgRequest);
    var response = runner.Run();
    return Results.Stream(async s =>
    {
        using (response)
        {
            await MessagePackSerializer.SerializeAsync(s, response, options, token);
        }
    }, contentType);
})
.WithName("Run")
.Accepts<PulseGenRequest>(contentType)
.Produces<PulseGenResponse>(StatusCodes.Status200OK, contentType)
.Produces(StatusCodes.Status400BadRequest);

app.Run();
