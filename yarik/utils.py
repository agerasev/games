from typing import Tuple
from random import Random
from pygame import Vector2, Rect


def expand(
    rect: Tuple[float, float] | Vector2 | Rect,
    value: float | Tuple[float, float] | Vector2,
) -> Rect:
    "Expand `rect` by `value`"

    if not isinstance(rect, Rect):
        rect = Rect(rect, (0, 0))

    if isinstance(value, float):
        value = Vector2(value, value)
    value = Vector2(value)

    return Rect(rect.topleft - value, rect.size + 2 * value)


def clamp(pos: Vector2, rect: Rect) -> Vector2:
    "Clamp `pos` inside `rect`"
    return Vector2(
        max(rect.left, min(pos.x, rect.right)),
        max(rect.top, min(pos.y, rect.bottom)),
    )


def random_uniform(rng: Random, rect: Rect) -> Vector2:
    "Uniformly sample point inside `rect`"
    return Vector2(
        rng.uniform(rect.left, rect.right),
        rng.uniform(rect.top, rect.bottom),
    )
