using Microsoft.AspNetCore.SignalR;

namespace Qynit.PulseGen.Server.Hubs;

public class PlotHub : Hub<IPlotClient>
{
    public const string Uri = "/plot/hub";
}
