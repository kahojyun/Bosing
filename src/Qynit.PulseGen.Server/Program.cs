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

app.MapPost("/run", async (HttpContext context, CancellationToken token) =>
{
    if (context.Request.ContentType == "application/msgpack")
    {
        var request = await MessagePackSerializer.DeserializeAsync<PulseGenRequest>(context.Request.BodyReader.AsStream(), cancellationToken: token);
        var runner = new PulseGenRunner(request);
        var response = runner.Run();
        return Results.Stream(async s =>
        {
            using (response)
            {
                await MessagePackSerializer.SerializeAsync(s, response, options, token);
            }
        }, "application/msgpack");
    }
    return Results.BadRequest();
})
.WithName("Run")
.Accepts<PulseGenRequest>("application/msgpack")
.Produces<PulseGenResponse>(StatusCodes.Status200OK, "application/msgpack")
.Produces(StatusCodes.Status400BadRequest);

app.Run();
