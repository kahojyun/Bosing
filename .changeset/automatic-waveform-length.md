---
default: minor
---

# Support schedule-based automatic waveform lengths

Allow `Channel.length` to default to `None` and derive waveform lengths from the logical schedule
duration. Sampling bounds errors now report the required sample range and available waveform
length.
