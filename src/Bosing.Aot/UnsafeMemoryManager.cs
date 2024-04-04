using System.Buffers;

namespace Bosing.Aot;

internal unsafe class UnsafeMemoryManager<T>(T* pointer, int length) : MemoryManager<T>
    where T : unmanaged
{
    public override Span<T> GetSpan()
    {
        return new(pointer, length);
    }

    public override MemoryHandle Pin(int elementIndex = 0)
    {
        return new(pointer + elementIndex);
    }

    public override void Unpin() { }

    protected override void Dispose(bool disposing) { }
}
