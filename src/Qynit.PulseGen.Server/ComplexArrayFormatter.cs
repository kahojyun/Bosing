using System.Runtime.InteropServices;

using MessagePack;
using MessagePack.Formatters;

using Qynit.PulseGen.Server.Models;

namespace Qynit.PulseGen.Server;

public sealed class ComplexArrayFormatter : IMessagePackFormatter<PooledComplexArray>
{
    public PooledComplexArray Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options)
    {
        throw new NotImplementedException();
    }

    public void Serialize(ref MessagePackWriter writer, PooledComplexArray value, MessagePackSerializerOptions options)
    {
        if (value == null)
        {
            writer.WriteNil();
            return;
        }

        switch (value)
        {
            case PooledComplexArray<float> floatArray:
                WriteArray(ref writer, floatArray, options);
                break;
            case PooledComplexArray<double> doubleArray:
                WriteArray(ref writer, doubleArray, options);
                break;
            default:
                break;
        }
    }

    private static void WriteArray<T>(ref MessagePackWriter writer, PooledComplexArray<T> value, MessagePackSerializerOptions options) where T : unmanaged
    {
        writer.WriteArrayHeader(3);
        if (typeof(T) == typeof(float))
        {
            writer.Write((byte)DataType.Float32);
        }
        else if (typeof(T) == typeof(double))
        {
            writer.Write((byte)DataType.Float64);
        }
        else
        {
            throw new NotSupportedException();
        }
        writer.Write(value.IsReal);
        if (value.IsReal)
        {
            var bytesI = MemoryMarshal.AsBytes(value.DataI);
            var length = bytesI.Length;

            WriteBinHeader(ref writer, length);
            writer.WriteRaw(bytesI);
        }
        else
        {
            var bytesI = MemoryMarshal.AsBytes(value.DataI);
            var bytesQ = MemoryMarshal.AsBytes(value.DataQ);
            var length = bytesI.Length * 2;

            WriteBinHeader(ref writer, length);
            writer.WriteRaw(bytesI);
            writer.WriteRaw(bytesQ);
        }

    }

    private static void WriteBinHeader(ref MessagePackWriter writer, int length)
    {
        var span = writer.GetSpan(5);
        span[0] = MessagePackCode.Bin32;
        WriteBigEndian(length, span[1..]);
        writer.Advance(5);
    }

    private static void WriteBigEndian(int value, Span<byte> span)
    {
        unchecked
        {
            span[3] = (byte)value;
            span[2] = (byte)(value >> 8);
            span[1] = (byte)(value >> 16);
            span[0] = (byte)(value >> 24);
        }
    }
}
