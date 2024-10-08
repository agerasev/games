from __future__ import annotations

from typing import Any, overload
from random import Random

from pygame import Vector2, Rect


class Rect2:
    "Real-valued Rect"

    xy: Vector2
    wh: Vector2

    @property
    def left(self) -> float:
        return self.xy.x

    @property
    def top(self) -> float:
        return self.xy.y

    @property
    def right(self) -> float:
        return self.xy.x + self.wh.x

    @property
    def bottom(self) -> float:
        return self.xy.y + self.wh.y

    @property
    def center(self) -> Vector2:
        return self.xy + 0.5 * self.wh

    @overload
    def __init__(self, x: float, y: float, w: float, h: float) -> None: ...
    @overload
    def __init__(
        self, xy: Vector2 | tuple[float, float], wh: Vector2 | tuple[float, float]
    ) -> None: ...
    @overload
    def __init__(self, rect: Rect | Rect2) -> None: ...

    def __init__(self, *args: Any, **kws: Any) -> None:
        if len(args) == 1:
            rect = args[0]
            if isinstance(rect, Rect):
                self.xy = Vector2(rect.x, rect.y)
                self.wh = Vector2(rect.w, rect.h)
            elif isinstance(rect, Rect2):
                self.xy = rect.xy
                self.wh = rect.wh
            else:
                raise TypeError()
        elif len(args) == 2:
            xy, wh = args
            self.xy = Vector2(xy)
            self.wh = Vector2(wh)
        elif len(args) == 4:
            x, y, w, h = args
            self.xy = Vector2(x, y)
            self.wh = Vector2(w, h)
        else:
            raise TypeError()


def expand(
    rect: tuple[float, float] | Vector2 | Rect | Rect2,
    value: float | tuple[float, float] | Vector2,
) -> Rect2:
    "Expand `rect` by `value`"

    if isinstance(rect, Rect2):
        pass
    elif isinstance(rect, Rect):
        rect = Rect2(rect)
    else:
        rect = Rect2(rect, (0, 0))

    if isinstance(value, float):
        value = Vector2(value, value)
    value = Vector2(value)

    return Rect2(rect.xy - value, rect.wh + 2 * value)


def clamp(pos: Vector2, rect: Rect2) -> Vector2:
    "Clamp `pos` inside `rect`"
    return Vector2(
        max(rect.left, min(pos.x, rect.right)),
        max(rect.top, min(pos.y, rect.bottom)),
    )


def random_uniform(rng: Random, rect: Rect2) -> Vector2:
    "Uniformly sample point inside `rect`"
    return Vector2(
        rng.uniform(rect.left, rect.right),
        rng.uniform(rect.top, rect.bottom),
    )
