from dataclasses import dataclass

import pygame
from pygame import Vector2, Surface


@dataclass
class ScaledCache:
    image: Surface
    size: Vector2


class Image:
    "Image caching scaled version of itself"

    def __init__(self, image: Surface) -> None:
        self.image = image
        self.scaled: ScaledCache | None = None

    def size(self) -> Vector2:
        return Vector2(self.image.get_size())

    def scale(self, size: tuple[float, float] | Vector2) -> Surface:
        size = Vector2(size)

        if self.scaled is not None:
            if self.scaled.size != size:
                self.scaled = None

        if self.scaled is None:
            self.scaled = ScaledCache(pygame.transform.scale(self.image, size), size)

        return self.scaled.image

    def scale_by(self, scale: float) -> Surface:
        return self.scale(self.size() * scale)
