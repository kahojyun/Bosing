using MessagePack;

using Python.Runtime;

using Qynit.PulseGen.Server.Models;
using Qynit.PulseGen.Server.Services;

namespace Qynit.PulseGen.Server;

public static class PythonApi
{
    internal static Server? ServerInstance;
    public static void Run(PyObject inRequestMsg, PyObject outWaveforms)
    {
        using (Py.GIL())
        using (inRequestMsg)
        using (outWaveforms)
        {
            var request = DeserializeRequest(inRequestMsg);
            var waveforms = GenerateWaveforms(request);
            using var pyDict = new PyDict(outWaveforms);
            CopyToPyWaveformDict(pyDict, request, waveforms);
            TryUpdatePlots(request, waveforms);
        }
    }

    public static void StartServer()
    {
        if (ServerInstance is null)
        {
            ServerInstance = Server.CreateApp([], true);
            ServerInstance.Start();
        }
    }

    public static void StopServer()
    {
        if (ServerInstance is not null)
        {
            ServerInstance.Stop();
            ServerInstance.Dispose();
            ServerInstance = null;
        }
    }

    private static void TryUpdatePlots(ScheduleRequest request, List<PooledComplexArray<float>> waveforms)
    {
        if (ServerInstance?.GetPlotService() is IPlotService service)
        {
            var arcWaveforms = waveforms.Select(ArcUnsafe.Wrap).ToList();
            service.UpdatePlots(request.ChannelTable!.Zip(arcWaveforms).ToDictionary(x => x.First.Name, x => new PlotData(x.First.Name, x.Second, 1.0 / x.First.SampleRate)));
        }
        else
        {
            foreach (var waveform in waveforms)
            {
                waveform.Dispose();
            }
        }
    }

    private static List<PooledComplexArray<float>> GenerateWaveforms(ScheduleRequest request)
    {
        var state = PythonEngine.BeginAllowThreads();
        try
        {
            var runner = new ScheduleRunner(request);
            return runner.Run();
        }
        finally
        {
            PythonEngine.EndAllowThreads(state);
        }
    }

    private static void CopyToPyWaveformDict(PyDict outWaveforms, ScheduleRequest request, List<PooledComplexArray<float>> waveforms)
    {
        foreach (var (channel, waveform) in request.ChannelTable!.Zip(waveforms))
        {
            var chName = channel.Name;
            using var arrayTuple = outWaveforms[chName];
            using var iArrayObject = arrayTuple[0];
            CopyToPyBuffer(waveform.DataI, chName, iArrayObject);
            using var qArrayObject = arrayTuple[1];
            CopyToPyBuffer(waveform.DataQ, chName, qArrayObject);
        }
    }

    private static unsafe ScheduleRequest DeserializeRequest(PyObject inRequestMsg)
    {
        using var requestMsg = inRequestMsg.GetBuffer();
        if (requestMsg.ItemSize != 1 || requestMsg.Dimensions != 1)
        {
            throw new ArgumentException("Message must be a byte array");
        }
        var msgLength = requestMsg.Length;
        using var stream = new UnmanagedMemoryStream((byte*)requestMsg.Buffer, msgLength);
        var options = Server.MessagePackSerializerOptions;
        return MessagePackSerializer.Deserialize<ScheduleRequest>(stream, options);
    }

    private static unsafe void CopyToPyBuffer(ReadOnlySpan<float> waveform, string chName, PyObject arrayObject)
    {
        using var pyBuffer = arrayObject.GetBuffer();
        if (pyBuffer.ItemSize != sizeof(float) || pyBuffer.Length != waveform.Length * sizeof(float))
        {
            throw new ArgumentException($"Waveform {chName} has wrong shape");
        }
        var span = new Span<float>((float*)pyBuffer.Buffer, waveform.Length);
        waveform.CopyTo(span);
    }
}
