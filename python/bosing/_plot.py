from __future__ import annotations

import logging
from typing import TYPE_CHECKING

import matplotlib.pyplot as plt
from matplotlib.collections import PatchCollection
from matplotlib.patches import Rectangle

if TYPE_CHECKING:
    from collections.abc import Iterator

    from matplotlib.axes import Axes

    from bosing._bosing import PlotItem

__all__ = ["plot"]

logger = logging.getLogger(__name__)


def plot(ax: Axes | None, blocks: Iterator[PlotItem]) -> Axes:
    if ax is None:
        ax = plt.gca()
    patches = [Rectangle((x.start, x.depth), x.span, 1) for x in blocks]
    collection = PatchCollection(patches)
    _ = ax.add_collection(collection)
    return ax
