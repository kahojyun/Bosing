using Microsoft.AspNetCore.Components;

namespace Qynit.PulseGen.Server;

internal class JsLocation
{
    public const string Root = "./dist";

    public static string GetPath<T>() where T : ComponentBase
    {
        return PathCache<T>.Path;
    }

    private class PathCache<T>
    {
        public static readonly string Path = GetPath();
        private static string GetPath()
        {
            var type = typeof(T);
            var rootNamespace = typeof(JsLocation).Namespace!;
            var path = type.FullName!.Replace(rootNamespace, "").Replace(".", "/").TrimStart('/');
            return $"{Root}/{path}.js";
        }
    }
}
