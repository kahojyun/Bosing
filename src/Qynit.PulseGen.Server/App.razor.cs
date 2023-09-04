using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;

namespace Qynit.PulseGen.Server;

public sealed partial class App : IDisposable
{
    [Inject]
    public IJSRuntime JS { get; set; } = default!;

    private bool _isDarkMode = false;
    private DotNetObjectReference<App>? _objRef;

    private float BaseLayerLuminance => _isDarkMode ? 0.23f : 1f;

    protected override async Task OnAfterRenderAsync(bool firstRender)
    {
        if (firstRender)
        {
            await using var module = await JS.ImportComponentModule<App>();
            _objRef = DotNetObjectReference.Create(this);
            await module.InvokeVoidAsync("init", _objRef);
            _isDarkMode = await module.InvokeAsync<bool>("isSystemDarkMode");
            StateHasChanged();
        }
    }

    [JSInvokable]
    public async ValueTask OnDarkModeChanged(bool isDarkMode)
    {
        _isDarkMode = isDarkMode;
        await InvokeAsync(StateHasChanged);
    }

    public void Dispose()
    {
        _objRef?.Dispose();
    }
}
