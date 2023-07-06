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
        var nBytes = sizeof(double) * value.Length;
        writer.WriteBinHeader(nBytes);
        writer.WriteRaw(MemoryMarshal.AsBytes(value.DataI));
        writer.WriteBinHeader(nBytes);
        writer.WriteRaw(MemoryMarshal.AsBytes(value.DataQ));
    }
}
