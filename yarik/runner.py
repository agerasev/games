from typing import Callable
from dataclasses import dataclass

import pygame
from pygame import Surface
from pygame.event import Event


@dataclass
class Context:
    screen: Surface
    events: list[Event]
    dt: float


class Game:
    def step(self, cx: Context) -> None:
        raise NotImplementedError()


def run(make_game: Callable[[], Game], initial_window_size: tuple[int, int]) -> None:
    pygame.init()

    screen = pygame.display.set_mode(initial_window_size, pygame.RESIZABLE)

    game = make_game()

    clock = pygame.time.Clock()
    dt = 0.0

    running = True
    while running:
        events = pygame.event.get()
        for event in events:
            if event.type == pygame.QUIT:
                running = False
            if event.type == pygame.VIDEORESIZE:
                screen = pygame.display.set_mode(event.size, pygame.RESIZABLE)

        cx = Context(screen, events, dt)
        try:
            game.step(cx)
        except StopIteration:
            break

        pygame.display.flip()
        dt = clock.tick(60) / 1000

    pygame.quit()
