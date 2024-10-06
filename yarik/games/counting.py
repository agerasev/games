from dataclasses import dataclass
from random import Random
import pygame as pg
from yarik import utils, Exit


def main():
    viewport = pg.Vector2(1280, 720)

    pg.init()
    screen = pg.display.set_mode((viewport.x, viewport.y))

    items = [
        pg.transform.scale_by(pg.image.load(path).convert_alpha(), 4)
        for path in ["assets/apple.png"]
    ]
    padding = 10

    font = pg.font.Font(None, 160)
    equals = font.render("=", True, "white")

    rng = Random()

    def play_game():
        clock = pg.time.Clock()
        dt = 0.0
        timeout = 1.0

        number = rng.randrange(1, 10)
        item = rng.choice(items)

        while True:
            for event in pg.event.get():
                if event.type == pg.QUIT:
                    raise Exit()
                if event.type == pg.KEYDOWN:
                    n = {
                        pg.K_0: 0,
                        pg.K_1: 1,
                        pg.K_2: 2,
                        pg.K_3: 3,
                        pg.K_4: 4,
                        pg.K_5: 5,
                        pg.K_6: 6,
                        pg.K_7: 7,
                        pg.K_8: 8,
                        pg.K_9: 9,
                    }.get(event.key)
                    if n is not None:
                        number = n
                    else:
                        if event.key == pg.K_MINUS:
                            number = max(0, number - 1)
                        elif event.key == pg.K_EQUALS:
                            number = min(10, number + 1)

            screen.fill("black")
            width = padding * (number + number // 5) + item.get_width() * number
            for i in range(0, number):
                screen.blit(
                    item,
                    (
                        viewport.x / 2
                        - width / 2
                        + padding * (i + 2 * (i // 5))
                        + item.get_width() * i,
                        viewport.y / 4 - item.get_height() / 2,
                    ),
                )
            screen.blit(equals, viewport / 2 - pg.Vector2(equals.get_size()) / 2)

            text = font.render(f"{number}", True, "white")
            screen.blit(
                text,
                pg.Vector2(viewport.x / 2, 3 * viewport.y / 4)
                - pg.Vector2(text.get_size()) / 2,
            )

            if len(items) == 0:
                if timeout > 0.0:
                    timeout -= dt
                else:
                    break

            pg.display.flip()

            dt = clock.tick(60) / 1000

        return True

    while True:
        try:
            play_game()
        except Exit:
            break
    pg.quit()


if __name__ == "__main__":
    main()
