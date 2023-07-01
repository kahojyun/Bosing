namespace Qynit.Pulsewave;
internal record EnvelopeCacheKey<T>(EnvelopeInfo EnvelopeInfo, Envelope Envelope) where T : unmanaged;

