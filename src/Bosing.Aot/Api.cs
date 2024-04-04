using System.Runtime.InteropServices;

using MessagePack;

using Bosing.Aot.Models;

namespace Bosing.Aot;

public static class Api
{
    private static MessagePackSerializerOptions MessagePackSerializerOptions { get; } =
        new MessagePackSerializerOptions(GeneratedMessagePackResolver.InstanceWithStandardAotResolver);

    enum ErrorCode
    {
        Success = 0,
        DeserializeError = 1,
        GenerateWaveformsError = 2,
        KeyNotFound = 3,
        CopyWaveformError = 4,
        InvalidHandle = 5,
        InternalError = 6,
    }

    [UnmanagedCallersOnly(EntryPoint = "Bosing_Run")]
    public static unsafe int Run(byte* requestMsg, int requestMsgLen, void** outWaveformDict)
    {
        try
        {
            ScheduleRequest request;
            try
            {
                request = DeserializeRequest(requestMsg, requestMsgLen);
            }
            catch (Exception e)
            {
                Console.Error.WriteLine(e);
                return (int)ErrorCode.DeserializeError;
            }
            List<PooledComplexArray<float>> waveforms;
            try
            {
                waveforms = GenerateWaveforms(request);
            }
            catch (Exception e)
            {
                Console.Error.WriteLine(e);
                return (int)ErrorCode.GenerateWaveformsError;
            }
            var waveformsDict = waveforms.Zip(request.ChannelTable!).ToDictionary(x => x.Second.Name, x => x.First);
            var handle = GCHandle.Alloc(waveformsDict);
            *outWaveformDict = GCHandle.ToIntPtr(handle).ToPointer();
            return (int)ErrorCode.Success;
        }
        catch (Exception e)
        {
            Console.Error.WriteLine(e);
            return (int)ErrorCode.InternalError;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "Bosing_CopyWaveform")]
    public static unsafe int CopyWaveform(IntPtr handle, IntPtr chName, float* bufferI, float* bufferQ, int bufferLen)
    {
        try
        {
            Dictionary<string, PooledComplexArray<float>> waveformsDict;
            try
            {
                waveformsDict = (Dictionary<string, PooledComplexArray<float>>)GCHandle.FromIntPtr(handle).Target!;
            }
            catch (Exception e)
            {
                Console.Error.WriteLine(e);
                return (int)ErrorCode.InvalidHandle;
            }
            var chNameStr = Marshal.PtrToStringUTF8(chName);
            if (chNameStr is null)
            {
                return (int)ErrorCode.KeyNotFound;
            }
            if (!waveformsDict.TryGetValue(chNameStr, out var waveform))
            {
                return (int)ErrorCode.KeyNotFound;
            }
            try
            {
                var spanI = new Span<float>(bufferI, bufferLen);
                waveform.DataI.CopyTo(spanI);
                var spanQ = new Span<float>(bufferQ, bufferLen);
                waveform.DataQ.CopyTo(spanQ);
            }
            catch (Exception e)
            {
                Console.Error.WriteLine(e);
                return (int)ErrorCode.CopyWaveformError;
            }
            return (int)ErrorCode.Success;
        }
        catch (Exception e)
        {
            Console.Error.WriteLine(e);
            return (int)ErrorCode.InternalError;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "Bosing_FreeWaveform")]
    public static int FreeWaveform(IntPtr handle)
    {
        try
        {
            Dictionary<string, PooledComplexArray<float>> waveformsDict;
            try
            {
                waveformsDict = (Dictionary<string, PooledComplexArray<float>>)GCHandle.FromIntPtr(handle).Target!;
            }
            catch (Exception e)
            {
                Console.Error.WriteLine(e);
                return (int)ErrorCode.InvalidHandle;
            }
            foreach (var waveform in waveformsDict.Values)
            {
                waveform.Dispose();
            }
            GCHandle.FromIntPtr(handle).Free();
            return (int)ErrorCode.Success;
        }
        catch (Exception e)
        {
            Console.Error.WriteLine(e);
            return (int)ErrorCode.InternalError;
        }
    }

    private static List<PooledComplexArray<float>> GenerateWaveforms(ScheduleRequest request)
    {
        var runner = new ScheduleRunner(request);
        return runner.Run();
    }

    private static unsafe ScheduleRequest DeserializeRequest(byte* msg, int len)
    {
        using var memoryManager = new UnsafeMemoryManager<byte>(msg, len);
        var memory = memoryManager.Memory;
        var options = MessagePackSerializerOptions;
        var formatter = options.Resolver.GetFormatterWithVerify<ScheduleRequest>();
        var reader = new MessagePackReader(memory);
        return formatter.Deserialize(ref reader, options);
    }
}
