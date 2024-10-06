from random import Random
import pygame as pg
from . import utils


def main():
    viewport = pg.Vector2(1280, 720)

    pg.init()
    screen = pg.display.set_mode((viewport.x, viewport.y))

    mouse = pg.transform.scale_by(pg.image.load("assets/mouse.png").convert_alpha(), 2)
    cheese = pg.transform.scale_by(
        pg.image.load("assets/cheese.png").convert_alpha(), 2
    )

    running = True

    def play_game():
        clock = pg.time.Clock()
        dt = 0.0
        timeout = 1.0

        player_pos = viewport / 2
        player_size = pg.Vector2(mouse.get_size())
        speed = 300.0

        rng = Random(0xDEADBEEF)
        item_size = pg.Vector2(cheese.get_size())
        items = [
            utils.random_uniform(rng, item_size / 2, viewport - item_size / 2)
            for i in range(16)
        ]

        while True:
            for event in pg.event.get():
                if event.type == pg.QUIT:
                    running = False
                    break

            screen.fill("black")
            for item_pos in items:
                screen.blit(cheese, item_pos - item_size / 2)
            screen.blit(mouse, player_pos - player_size / 2)

            keys = pg.key.get_pressed()
            if keys[pg.K_UP] or keys[pg.K_w]:
                player_pos.y -= speed * dt
            if keys[pg.K_DOWN] or keys[pg.K_s]:
                player_pos.y += speed * dt
            if keys[pg.K_LEFT] or keys[pg.K_a]:
                player_pos.x -= speed * dt
            if keys[pg.K_RIGHT] or keys[pg.K_d]:
                player_pos.x += speed * dt

            player_pos = utils.clamp(
                player_pos, player_size / 2, viewport - player_size / 2
            )

            items = [
                item_pos
                for item_pos in items
                if not utils.overlap(
                    player_pos, item_pos, player_size / 3, item_size / 3
                )
            ]
            if len(items) == 0:
                if timeout > 0.0:
                    timeout -= dt
                else:
                    break

            pg.display.flip()

            dt = clock.tick(60) / 1000

    while running:
        play_game()
    pg.quit()


if __name__ == "__main__":
    main()
