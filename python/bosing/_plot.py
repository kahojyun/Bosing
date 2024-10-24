from __future__ import annotations

from collections import defaultdict
from typing import TYPE_CHECKING

import matplotlib.pyplot as plt
import numpy as np
from matplotlib.patches import PathPatch
from matplotlib.path import Path
from matplotlib.ticker import EngFormatter

from bosing._bosing import ItemKind

if TYPE_CHECKING:
    from collections.abc import Iterator, Sequence

    from matplotlib.axes import Axes
    from typing_extensions import TypeAlias

    from bosing._bosing import PlotItem

    _RECTS: TypeAlias = defaultdict[ItemKind, list[tuple[float, float, float]]]
    _MARKERS: TypeAlias = defaultdict[ItemKind, tuple[list[float], list[float]]]


COLORS = {
    ItemKind.Play: "blue",
    ItemKind.ShiftPhase: "green",
    ItemKind.SetPhase: "red",
    ItemKind.ShiftFreq: "green",
    ItemKind.SetFreq: "red",
    ItemKind.SwapPhase: "purple",
    ItemKind.Barrier: "gray",
    ItemKind.Repeat: "yellow",
    ItemKind.Stack: "orange",
    ItemKind.Absolute: "cyan",
    ItemKind.Grid: "black",
}

MARKERS = {
    ItemKind.ShiftPhase: "$\\circlearrowleft$",
    ItemKind.SetPhase: "$\\circlearrowleft$",
    ItemKind.ShiftFreq: "$\\Uparrow$",
    ItemKind.SetFreq: "$\\Uparrow$",
    ItemKind.SwapPhase: "$\\leftrightarrow$",
}

LABELS = {
    ItemKind.Play: "Play",
    ItemKind.ShiftPhase: "Shift Phase",
    ItemKind.SetPhase: "Set Phase",
    ItemKind.ShiftFreq: "Shift Frequency",
    ItemKind.SetFreq: "Set Frequency",
    ItemKind.SwapPhase: "Swap Phase",
    ItemKind.Barrier: "Barrier",
    ItemKind.Repeat: "Repeat",
    ItemKind.Stack: "Stack",
    ItemKind.Absolute: "Absolute",
    ItemKind.Grid: "Grid",
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
) -> tuple[_RECTS, _MARKERS]:
    ch_stack: list[list[str]] = []
    rects: _RECTS = defaultdict(list)
    markers: _MARKERS = defaultdict(lambda: ([], []))

    for x in blocks:
        manage_channel_stack(ch_stack, x)
        if x.depth >= max_depth:
            continue
        for c in get_plot_channels(ch_stack, x, channels):
            if c in channels_ystart:
                y = channels_ystart[c] + x.depth
                if x.kind in MARKERS:
                    mx, my = markers[x.kind]
                    mx.append(x.start)
                    my.append(y)
                else:
                    rects[x.kind].append((x.start, y, x.span))
    return rects, markers


def plot(
    ax: Axes | None, blocks: Iterator[PlotItem], channels: Sequence[str], max_depth: int
) -> Axes:
    if ax is None:
        ax = plt.gca()

    channels_ystart = {c: i * (max_depth + 1) for i, c in enumerate(channels)}
    rects, markers = process_blocks(blocks, channels, max_depth, channels_ystart)

    for k, r in rects.items():
        # numrects x [x, y, width]
        r_arr = np.array(r)
        # numrects x numsides x 2
        xy = np.empty((r_arr.shape[0], 4, 2))
        xy[:, :2, 0] = r_arr[:, np.newaxis, 0]
        xy[:, 2:, 0] = r_arr[:, np.newaxis, 0] + r_arr[:, np.newaxis, 2]
        xy[:, [0, 3], 1] = r_arr[:, np.newaxis, 1]
        xy[:, [1, 2], 1] = r_arr[:, np.newaxis, 1] + 1
        path = Path.make_compound_path_from_polys(xy)
        patch = PathPatch(path)
        patch.set_facecolor(COLORS[k])
        patch.set_label(LABELS[k])
        _ = ax.add_patch(patch)

    for k, (mx, my) in markers.items():
        _ = ax.plot(  # pyright: ignore[reportUnknownMemberType]
            mx,
            my,
            linestyle="",
            marker=MARKERS[k],
            color=COLORS[k],
            label=LABELS[k],
            markersize=12,
        )

    _ = ax.set_yticks(list(channels_ystart.values()), channels_ystart.keys())  # pyright: ignore[reportUnknownMemberType]
    ax.xaxis.set_major_formatter(EngFormatter(places=3))
    _ = ax.set_xlabel("Time")  # pyright: ignore[reportUnknownMemberType]
    _ = ax.set_ylabel("Channels")  # pyright: ignore[reportUnknownMemberType]
    _ = ax.legend()  # pyright: ignore[reportUnknownMemberType]
    ax.autoscale()

    return ax
