# pyright: reportAny=false
# pyright: reportUnknownArgumentType=false
# pyright: reportUnknownVariableType=false
from __future__ import annotations

from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    import contextlib

    with contextlib.suppress(ImportError):
        from rich.repr import RichReprResult

    class _IRichRepr(Protocol):
        def __rich_repr__(self) -> RichReprResult: ...


def repr_from_rich(obj: _IRichRepr) -> str:
    parts: list[str] = []
    for tpls in obj.__rich_repr__():
        if isinstance(tpls, tuple):
            if len(tpls) == 1:
                value = tpls[0]
                parts.append(repr(value))
            elif len(tpls) == 2:  # noqa: PLR2004
                key, value = tpls
                parts.append(f"{key}={value!r}")
            elif len(tpls) == 3:  # noqa: PLR2004
                key, value, default = tpls
                if value != default:
                    parts.append(f"{key}={value!r}")
            else:
                msg = "Invalid tuple length"
                raise ValueError(msg)
        else:
            parts.append(repr(tpls))
    return f"{obj.__class__.__name__}({', '.join(parts)})"
