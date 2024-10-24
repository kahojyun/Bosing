from __future__ import annotations

import logging
from collections import defaultdict
from typing import TYPE_CHECKING

import matplotlib.pyplot as plt
from matplotlib.collections import PatchCollection
from matplotlib.patches import Patch, Rectangle
from matplotlib.ticker import EngFormatter

from bosing._bosing import ItemKind

if TYPE_CHECKING:
    from collections.abc import Iterator, Sequence

    from matplotlib.axes import Axes

    from bosing._bosing import PlotItem

logger = logging.getLogger(__name__)

FACECOLORS = {
    ItemKind.Play: "blue",
    ItemKind.ShiftPhase: "green",
    ItemKind.SetPhase: "green",
    ItemKind.ShiftFreq: "red",
    ItemKind.SetFreq: "red",
    ItemKind.SwapPhase: "purple",
    ItemKind.Barrier: "gray",
    ItemKind.Repeat: "yellow",
    ItemKind.Stack: "orange",
    ItemKind.Absolute: "cyan",
    ItemKind.Grid: "black",
}


def manage_channel_stack(ch_stack: list[list[str]], x: PlotItem) -> None:
    prev_depth = len(ch_stack) - 1
    if x.depth > prev_depth:
        ch_stack.append(x.channels)
    elif x.depth < prev_depth:
        _ = ch_stack.pop()
        ch_stack[-1] = x.channels
    else:
        ch_stack[-1] = x.channels


def get_plot_channels(
    ch_stack: list[list[str]], x: PlotItem, channels: Sequence[str]
) -> Sequence[str]:
    if x.kind == ItemKind.Barrier and len(x.channels) == 0:
        for chs in reversed(ch_stack):
            if len(chs) > 0:
                return chs
        return channels
    return x.channels


def process_blocks(
    blocks: Iterator[PlotItem],
    channels: Sequence[str],
    max_depth: int,
    channels_ystart: dict[str, int],
) -> defaultdict[ItemKind, list[Patch]]:
    ch_stack: list[list[str]] = []
    patches: defaultdict[ItemKind, list[Patch]] = defaultdict(list)

    for x in blocks:
        manage_channel_stack(ch_stack, x)
        if x.depth >= max_depth:
            continue
        for c in get_plot_channels(ch_stack, x, channels):
            if c in channels_ystart:
                y = channels_ystart[c] + x.depth
                patches[x.kind].append(Rectangle((x.start, y), x.span, 1))
    return patches


def plot(
    ax: Axes | None, blocks: Iterator[PlotItem], channels: Sequence[str], max_depth: int
) -> Axes:
    if ax is None:
        ax = plt.gca()

    channels_ystart = {c: i * (max_depth + 1) for i, c in enumerate(channels)}
    patches = process_blocks(blocks, channels, max_depth, channels_ystart)

    for k, p in patches.items():
        collection = PatchCollection(p)
        collection.set_facecolor(FACECOLORS[k])
        collection.set_edgecolor("black")
        _ = ax.add_collection(collection)

    _ = ax.set_yticks(list(channels_ystart.values()), channels_ystart.keys())  # pyright: ignore[reportUnknownMemberType]
    ax.xaxis.set_major_formatter(EngFormatter(places=3))
    _ = ax.set_xlabel("Time")  # pyright: ignore[reportUnknownMemberType]
    _ = ax.set_ylabel("Channels")  # pyright: ignore[reportUnknownMemberType]
    ax.autoscale()

    return ax
