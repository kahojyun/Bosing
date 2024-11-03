from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterable
    from typing import Protocol

    from typing_extensions import TypeAlias

    TupleRichReprResult: TypeAlias = Iterable[
        "tuple[object] | tuple[str, object] | tuple[str, object, object]"
    ]

    class IRichRepr(Protocol):
        def __rich_repr__(self) -> TupleRichReprResult: ...


def repr_from_rich(obj: IRichRepr) -> str:
    parts: list[str] = []
    for tpls in obj.__rich_repr__():
        if len(tpls) == 1:
            value = tpls[0]
            parts.append(repr(value))
        elif len(tpls) == 2:  # noqa: PLR2004
            key, value = tpls
            parts.append(f"{key}={value!r}")
        else:
            key, value, default = tpls
            if value != default:
                parts.append(f"{key}={value!r}")
    return f"{obj.__class__.__name__}({', '.join(parts)})"
