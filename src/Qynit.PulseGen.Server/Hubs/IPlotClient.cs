namespace Qynit.PulseGen.Server.Hubs;

public interface IPlotClient
{
    Task ReceiveNames(IEnumerable<string> names);
}
