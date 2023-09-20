using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;

namespace Qynit.PulseGen.Server.Shared;

public sealed partial class TriCheckbox : IAsyncDisposable
{
    [Parameter]
    public RenderFragment? ChildContent { get; set; }

    [Parameter]
    public bool Value { get; set; }

    [Parameter]
    public EventCallback<bool> ValueChanged { get; set; }

    [Parameter]
    public bool Indeterminate { get; set; }

    [Inject]
    private IJSRuntime JS { get; set; } = default!;

    private bool CurrentValue
    {
        get => Value;
        set
        {
            if (value != Value)
            {
                Value = value;
                _ = ValueChanged.InvokeAsync(value);
            }
        }
    }

    private IJSObjectReference? _module;
    private ElementReference? _checkbox;
    protected override async Task OnAfterRenderAsync(bool firstRender)
    {
        if (firstRender)
        {
            _module = await JS.ImportComponentModule<TriCheckbox>();
        }

        var checkbox = _checkbox;
        if (_module is not null && checkbox is not null)
        {
            await _module.InvokeVoidAsync("setIndeterminate", checkbox, Indeterminate);
        }
    }

    public async ValueTask DisposeAsync()
    {
        if (_module is not null)
        {
            try
            {
                await _module.DisposeAsync();
            }
            catch (JSDisconnectedException)
            {
            }
        }
    }
}
