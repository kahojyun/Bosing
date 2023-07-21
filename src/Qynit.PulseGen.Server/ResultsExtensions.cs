using System.Reflection;

using MessagePack;

using Microsoft.AspNetCore.Http.Metadata;
using Microsoft.AspNetCore.Mvc;

namespace Qynit.PulseGen.Server;

internal static class ResultsExtensions
{
    public static IResult MessagePack<T>(this IResultExtensions resultExtensions, T obj, MessagePackSerializerOptions? options = null)
    {
        ArgumentNullException.ThrowIfNull(resultExtensions);
        return new MessagePackResult<T>(obj, options);
    }
}

internal class MessagePackResult<T> : IResult, IEndpointMetadataProvider
{
    private readonly T _obj;
    private readonly MessagePackSerializerOptions? _options;

    public MessagePackResult(T obj, MessagePackSerializerOptions? options)
    {
        _obj = obj;
        _options = options;
    }

    public static void PopulateMetadata(MethodInfo method, EndpointBuilder builder)
    {
        builder.Metadata.Add(new ProducesResponseTypeAttribute(typeof(T), StatusCodes.Status200OK, "application/msgpack"));
    }

    public async Task ExecuteAsync(HttpContext httpContext)
    {
        httpContext.Response.ContentType = "application/msgpack";
        var body = httpContext.Response.Body;
        var requestAborted = httpContext.RequestAborted;
        await MessagePackSerializer.SerializeAsync(body, _obj, _options, requestAborted);
    }
}
