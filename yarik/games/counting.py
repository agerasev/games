from random import Random

import pygame
from pygame import Vector2

from yarik import Context, Game, run


class CountingGame(Game):
    def __init__(self) -> None:
        self.item = pygame.transform.scale_by(
            pygame.image.load("assets/apple.png").convert_alpha(), 4
        )
        self.padding = 10

        self.font = pygame.font.Font(None, 160)
        self.equals = self.font.render("=", True, "white")

        self.rng = Random()
        self.number = self.rng.randrange(1, 10)

    def step(self, cx: Context) -> None:
        screen = cx.screen
        viewport = Vector2(screen.get_size())

        number = self.number

        for event in cx.events:
            if event.type == pygame.KEYDOWN:
                n = {
                    pygame.K_0: 0,
                    pygame.K_1: 1,
                    pygame.K_2: 2,
                    pygame.K_3: 3,
                    pygame.K_4: 4,
                    pygame.K_5: 5,
                    pygame.K_6: 6,
                    pygame.K_7: 7,
                    pygame.K_8: 8,
                    pygame.K_9: 9,
                }.get(event.key)
                if n is not None:
                    number = n
                else:
                    if event.key == pygame.K_MINUS:
                        number = max(0, number - 1)
                    elif event.key == pygame.K_EQUALS:
                        number = min(10, number + 1)

        screen.fill("black")
        width = self.padding * (number + number // 5) + self.item.get_width() * number
        for i in range(0, number):
            screen.blit(
                self.item,
                (
                    viewport.x / 2
                    - width / 2
                    + self.padding * (i + 2 * (i // 5))
                    + self.item.get_width() * i,
                    viewport.y / 4 - self.item.get_height() / 2,
                ),
            )
        screen.blit(self.equals, viewport / 2 - Vector2(self.equals.get_size()) / 2)

        text = self.font.render(f"{number}", True, "white")
        screen.blit(
            text,
            Vector2(viewport.x / 2, 3 * viewport.y / 4) - Vector2(text.get_size()) / 2,
        )

        self.number = number


if __name__ == "__main__":
    run(CountingGame, (1280, 720))
