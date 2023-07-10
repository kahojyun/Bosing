﻿using MessagePack;

namespace Qynit.PulseGen.Server;

[MessagePackObject]
public sealed record PulseGenRequest(
    [property: Key(0)] IList<ChannelInfo> ChannelTable,
    [property: Key(1)] IList<ShapeInfo> ShapeTable,
    [property: Key(2)] IEnumerable<Instruction> Instructions);