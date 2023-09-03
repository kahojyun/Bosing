using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;

namespace Qynit.PulseGen.Server;

internal static class Extensions
{
    public static async ValueTask<IJSObjectReference> ImportComponentModule<T>(this IJSRuntime js) where T : ComponentBase
    {
        var path = JsLocation.GetPath<T>();
        return await js.InvokeAsync<IJSObjectReference>("import", path);
    }
}
