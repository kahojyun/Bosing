using System.Runtime.InteropServices;

using MessagePack;
using MessagePack.Formatters;

namespace Qynit.PulseGen.Server;

public sealed class ComplexArrayFormatter : IMessagePackFormatter<PooledComplexArray<double>>
{
    public PooledComplexArray<double> Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options)
    {
        throw new NotImplementedException();
    }

    public void Serialize(ref MessagePackWriter writer, PooledComplexArray<double> value, MessagePackSerializerOptions options)
    {
        if (value == null)
        {
            writer.WriteNil();
            return;
        }

        writer.WriteArrayHeader(2);
        var bytesI = MemoryMarshal.AsBytes(value.DataI);
        var bytesQ = MemoryMarshal.AsBytes(value.DataQ);
        var length = bytesI.Length;
        if (length <= ushort.MaxValue)
        {
            writer.Write(bytesI);
            writer.Write(bytesQ);
        }
        else
        {
            WriteBinHeader(ref writer, length);
            writer.WriteRaw(bytesI);
            WriteBinHeader(ref writer, length);
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
