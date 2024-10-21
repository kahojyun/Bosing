r"""Generates microwave pulses for superconducting quantum computing experiments.

.. caution::

    The unit of phase is number of cycles, not radians. For example, a phase
    of :math:`0.5` means a phase shift of :math:`\pi` radians.
"""

from ._bosing import *
from ._bosing import __all__ as __all__
