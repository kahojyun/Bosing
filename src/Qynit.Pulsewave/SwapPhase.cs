﻿namespace Qynit.Pulsewave;
public sealed record SwapPhase(double ReferenceTime, Channel Channel1, Channel Channel2) : Instruction;
