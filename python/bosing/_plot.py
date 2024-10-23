from __future__ import annotations

import logging
from typing import TYPE_CHECKING

import matplotlib.pyplot as plt
from matplotlib.collections import PatchCollection
from matplotlib.patches import Patch, Rectangle

from bosing._bosing import ItemKind

if TYPE_CHECKING:
    from collections.abc import Iterator

    from matplotlib.axes import Axes

    from bosing._bosing import PlotItem


logger = logging.getLogger(__name__)


def get_play_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_shiftphase_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_setphase_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_shiftfreq_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_setfreq_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_swapphase_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_barrier_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_repeat_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_stack_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_absolute_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


def get_grid_patch(item: PlotItem) -> Patch:
    return Rectangle((item.start, item.depth), item.span, 1)


DISPATCH = {
    ItemKind.Play: get_play_patch,
    ItemKind.ShiftPhase: get_shiftphase_patch,
    ItemKind.SetPhase: get_setphase_patch,
    ItemKind.ShiftFreq: get_shiftfreq_patch,
    ItemKind.SetFreq: get_setfreq_patch,
    ItemKind.SwapPhase: get_swapphase_patch,
    ItemKind.Barrier: get_barrier_patch,
    ItemKind.Repeat: get_repeat_patch,
    ItemKind.Stack: get_stack_patch,
    ItemKind.Absolute: get_absolute_patch,
    ItemKind.Grid: get_grid_patch,
}


def plot(ax: Axes | None, blocks: Iterator[PlotItem]) -> Axes:
    if ax is None:
        ax = plt.gca()
    channels: list[list[str]] = []
    patches: list[Patch] = []
    for x in blocks:
        prev_depth = len(channels) - 1
        if x.depth > prev_depth:
            channels.append(x.channels)
        elif x.depth < prev_depth:
            _ = channels.pop()
            channels[-1] = x.channels
        else:
            channels[-1] = x.channels
        patches.append(DISPATCH[x.kind](x))
    collection = PatchCollection(patches)
    _ = ax.add_collection(collection)
    return ax
