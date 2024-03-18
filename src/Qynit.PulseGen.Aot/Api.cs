using System.Runtime.InteropServices;

using MessagePack;
using MessagePack.Resolvers;

using Qynit.PulseGen.Aot.Models;

namespace Qynit.PulseGen.Aot;

public static class Api
{
    private static MessagePackSerializerOptions MessagePackSerializerOptions { get; } =
        new MessagePackSerializerOptions(GeneratedMessagePackResolver.InstanceWithStandardAotResolver);

    [UnmanagedCallersOnly(EntryPoint = "Qynit_PulseGen_Run")]
    public static unsafe IntPtr Run(byte* requestMsg, int requestMsgLen)
    {
        var request = DeserializeRequest(requestMsg, requestMsgLen);
        var waveforms = GenerateWaveforms(request);
        var waveformsDict = waveforms.Zip(request.ChannelTable!).ToDictionary(x => x.Second.Name, x => x.First);
        var handle = GCHandle.Alloc(waveformsDict);
        return GCHandle.ToIntPtr(handle);
    }

    [UnmanagedCallersOnly(EntryPoint = "Qynit_PulseGen_CopyWaveform")]
    public static unsafe void CopyWaveform(IntPtr handle, IntPtr chName, float* bufferI, float* bufferQ, int bufferLen)
    {
        try
        {
            var waveformsDict = (Dictionary<string, PooledComplexArray<float>>)GCHandle.FromIntPtr(handle).Target!;
            var chNameStr = Marshal.PtrToStringUTF8(chName);
            var waveform = waveformsDict[chNameStr];
            var spanI = new Span<float>(bufferI, bufferLen);
            waveform.DataI.CopyTo(spanI);
            var spanQ = new Span<float>(bufferQ, bufferLen);
            waveform.DataQ.CopyTo(spanQ);
        }
        catch (Exception e)
        {
            Console.WriteLine(e);
            throw;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "Qynit_PulseGen_FreeWaveform")]
    public static unsafe void FreeWaveform(IntPtr handle)
    {
        GCHandle.FromIntPtr(handle).Free();
    }

    [UnmanagedCallersOnly(EntryPoint = "Qynit_PulseGen_Hello")]
    public static void Hello()
    {
        throw new Exception("Hello from .NET");
    }

    private static List<PooledComplexArray<float>> GenerateWaveforms(ScheduleRequest request)
    {
        var runner = new ScheduleRunner(request);
        return runner.Run();
    }

    private static unsafe ScheduleRequest DeserializeRequest(byte* msg, int len)
    {
        using var stream = new UnmanagedMemoryStream(msg, len);
        var options = MessagePackSerializerOptions;
        return MessagePackSerializer.Deserialize<ScheduleRequest>(stream, options);
    }
}
