from random import Random
from pygame import Vector2


def clamp(pos: Vector2, a: Vector2, b: Vector2) -> Vector2:
    return Vector2(
        max(a.x, min(pos.x, b.x)),
        max(a.y, min(pos.y, b.y)),
    )


def overlap(a_pos: Vector2, b_pos: Vector2, a_size: Vector2, b_size: Vector2) -> bool:
    pos = a_pos - b_pos
    size = a_size + b_size
    return abs(pos.x) <= size.x and abs(pos.y) <= size.y


def random_uniform(rng: Random, a: Vector2, b: Vector2) -> Vector2:
    return Vector2(
        rng.uniform(a.x, b.x),
        rng.uniform(a.y, b.y),
    )
