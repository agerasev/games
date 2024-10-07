from dataclasses import dataclass
from random import Random

import pygame
from pygame import Rect, Vector2, Surface

from yarik import utils, Context, Game, run


@dataclass
class Player:
    pos: Vector2
    image: Surface
    size: Vector2
    speed: float


@dataclass
class Item:
    pos: Vector2
    image: Surface
    size: Vector2


class MouseGame(Game):
    def __init__(self) -> None:
        self.player_image = pygame.transform.scale_by(
            pygame.image.load("assets/mouse.png").convert_alpha(), 2
        )
        self.player_size = Vector2(self.player_image.get_size())

        items = [
            ("assets/cheese.png", 0.8),
            ("assets/apple.png", 0.2),
        ]
        self.item_images = [
            pygame.transform.scale_by(pygame.image.load(path).convert_alpha(), 2)
            for path, _ in items
        ]
        self.item_probs = [p for _, p in items]
        self.item_size = Vector2(self.item_images[0].get_size())
        assert all(
            [
                self.item_size == Vector2(item_image.get_size())
                for item_image in self.item_images
            ]
        )

        self.font = pygame.font.Font(None, 80)

        self.rng = Random()
        self.session: Session | None = None

    def step(self, cx: Context) -> None:
        if self.session is None:
            self.session = Session(self, cx.screen.get_rect())

        try:
            self.session.step(cx)
        except StopIteration:
            self.session = None


class Session(Game):
    def __init__(self, game: MouseGame, viewport: Rect, num_items: int = 16) -> None:
        self.viewport = viewport

        self.player = Player(
            pos=Vector2(viewport.center),
            image=game.player_image,
            size=game.player_size,
            speed=300.0,
        )

        self.items = [
            Item(
                pos=utils.random_uniform(
                    game.rng, utils.expand(viewport, -game.item_size / 2)
                ),
                image=game.rng.choices(game.item_images, game.item_probs)[0],
                size=game.item_size,
            )
            for _ in range(num_items)
        ]

        self.font = game.font

        self.timeout = 1.0
        self.counter = 0

    def step(self, cx: Context) -> None:
        player = self.player
        screen = cx.screen

        cx.screen.fill("black")
        for item in self.items:
            cx.screen.blit(item.image, item.pos - item.size / 2)
        cx.screen.blit(player.image, player.pos - player.size / 2)

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
            player.pos, utils.expand(self.viewport, -player.size / 2)
        )

        new_items = []
        for item in self.items:
            if Rect.colliderect(
                utils.expand(player.pos, player.size / 3),
                utils.expand(item.pos, item.size / 3),
            ):
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
    run(MouseGame, (1280, 720))
