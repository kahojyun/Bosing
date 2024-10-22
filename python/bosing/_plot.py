from __future__ import annotations

import logging
from typing import TYPE_CHECKING

import matplotlib.pyplot as plt
from matplotlib.collections import PatchCollection
from matplotlib.patches import Rectangle

if TYPE_CHECKING:
    from collections.abc import Iterable

    from matplotlib.axes import Axes

logger = logging.getLogger(__name__)


def plot(ax: Axes | None, blocks: Iterable[tuple[float, float]]) -> Axes:
    if ax is None:
        ax = plt.gca()
    patches = [Rectangle((t, 0), w, 1) for t, w in blocks]
    collection = PatchCollection(patches)
    _ = ax.add_collection(collection)
    return ax
