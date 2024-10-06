from dataclasses import dataclass
from random import Random
import pygame as pg
from . import utils


class Exit(Exception):
    pass


@dataclass
class Item:
    pos: pg.Vector2
    image: pg.Surface


def main():
    viewport = pg.Vector2(1280, 720)

    pg.init()
    screen = pg.display.set_mode((viewport.x, viewport.y))

    player_image = pg.transform.scale_by(
        pg.image.load("assets/mouse.png").convert_alpha(), 2
    )
    player_size = pg.Vector2(player_image.get_size())

    item_variants = [
        ("assets/cheese.png", 0.8),
        ("assets/apple.png", 0.2),
    ]
    item_images = [
        pg.transform.scale_by(pg.image.load(path).convert_alpha(), 2)
        for path, _ in item_variants
    ]
    item_probabilities = [p for _, p in item_variants]
    item_size = pg.Vector2(item_images[0].get_size())
    assert all(
        [item_size == pg.Vector2(item_image.get_size()) for item_image in item_images]
    )

    font = pg.font.Font(None, 80)

    rng = Random()

    def play_game():
        clock = pg.time.Clock()
        dt = 0.0
        timeout = 1.0
        counter = 0

        player_pos = viewport / 2
        speed = 300.0

        items = [
            Item(
                pos=utils.random_uniform(rng, item_size / 2, viewport - item_size / 2),
                image=rng.choices(item_images, item_probabilities)[0],
            )
            for i in range(16)
        ]

        while True:
            for event in pg.event.get():
                if event.type == pg.QUIT:
                    raise Exit()

            screen.fill("black")
            for item in items:
                screen.blit(item.image, item.pos - item_size / 2)
            screen.blit(player_image, player_pos - player_size / 2)

            text_surface = font.render(f"{counter}", True, "white")
            screen.blit(text_surface, (10, 10))

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

            new_items = []
            for item in items:
                if utils.overlap(player_pos, item.pos, player_size / 3, item_size / 3):
                    counter += 1
                else:
                    new_items.append(item)
            items = new_items

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
