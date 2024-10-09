from dataclasses import dataclass
from random import Random

import pygame
from pygame import Rect, Vector2, Surface

from yarik import utils, Context, Game, Image, run
from yarik.utils import Rect2


@dataclass
class Item:
    pos: Vector2
    image: Image
    radius: float

    @property
    def size(self) -> Vector2:
        return 2 * Vector2(self.radius, self.radius)


@dataclass
class Player(Item):
    speed: float


def blit_scaled(dst: Surface, src: Item, scale: float) -> None:
    dst.blit(src.image.scale(src.size * scale), (src.pos - src.size / 2) * scale)


class CollectingGame(Game):
    def __init__(self) -> None:
        self.player_image = Image(pygame.image.load("assets/mouse.png").convert_alpha())

        items = [
            ("assets/cheese.png", 0.8),
            ("assets/apple.png", 0.2),
        ]
        self.item_images = [
            Image(pygame.image.load(path).convert_alpha()) for path, _ in items
        ]
        self.item_probs = [p for _, p in items]

        self.font = pygame.font.Font(None, 80)

        self.rng = Random()
        self.session: Session | None = None

    def step(self, cx: Context) -> None:
        if self.session is None:
            self.session = Session(self, Vector2(42, 24))

        try:
            self.session.step(cx)
        except StopIteration:
            self.session = None


class Session(Game):
    def __init__(
        self, game: CollectingGame, map_size: Vector2, num_items: int = 16
    ) -> None:
        self.map_rect = Rect2((0, 0), map_size)

        self.player = Player(
            pos=self.map_rect.center,
            image=game.player_image,
            radius=1.0,
            speed=10.0,
        )

        item_radius = 0.5
        self.items = [
            Item(
                pos=utils.random_uniform(
                    game.rng, utils.expand(self.map_rect, -item_radius)
                ),
                image=game.rng.choices(game.item_images, game.item_probs)[0],
                radius=item_radius,
            )
            for _ in range(num_items)
        ]

        self.font = game.font

        self.timeout = 1.0
        self.counter = 0

    def step(self, cx: Context) -> None:
        scale = 30.0

        player = self.player
        screen = cx.screen

        cx.screen.fill("black")
        for item in self.items:
            blit_scaled(cx.screen, item, scale)
        blit_scaled(cx.screen, player, scale)

        text = self.font.render(f"{self.counter}", True, "white")
        cx.screen.blit(text, (10, 10))

        keys = pygame.key.get_pressed()
        if keys[pygame.K_UP] or keys[pygame.K_w]:
            player.pos.y -= player.speed * cx.dt
        if keys[pygame.K_DOWN] or keys[pygame.K_s]:
            player.pos.y += player.speed * cx.dt
        if keys[pygame.K_LEFT] or keys[pygame.K_a]:
            player.pos.x -= player.speed * cx.dt
        if keys[pygame.K_RIGHT] or keys[pygame.K_d]:
            player.pos.x += player.speed * cx.dt

        player.pos = utils.clamp(
            player.pos, utils.expand(self.map_rect, -player.radius)
        )

        new_items = []
        for item in self.items:
            if player.pos.distance_to(item.pos) < (player.radius + item.radius):
                self.counter += 1
            else:
                new_items.append(item)
        self.items = new_items

        if len(self.items) == 0:
            if self.timeout > 0.0:
                self.timeout -= cx.dt
            else:
                raise StopIteration()


if __name__ == "__main__":
    run(CollectingGame, (1280, 720))
