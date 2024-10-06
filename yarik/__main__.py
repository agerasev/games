# Example file showing a circle moving on screen
import pygame

width = 1280
height = 720

# pygame setup
pygame.init()
screen = pygame.display.set_mode((width, height))
clock = pygame.time.Clock()
running = True
dt = 0.0

pos = pygame.Vector2(screen.get_width() / 2, screen.get_height() / 2)
radius = 40.0
speed = 300.0

while running:
    # poll for events
    # pygame.QUIT event means the user clicked X to close your window
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            running = False

    # fill the screen with a color to wipe away anything from last frame
    screen.fill("black")

    pygame.draw.circle(screen, "red", pos, radius)

    keys = pygame.key.get_pressed()
    if keys[pygame.K_UP] or keys[pygame.K_w]:
        pos.y -= speed * dt
    if keys[pygame.K_DOWN] or keys[pygame.K_s]:
        pos.y += speed * dt
    if keys[pygame.K_LEFT] or keys[pygame.K_a]:
        pos.x -= speed * dt
    if keys[pygame.K_RIGHT] or keys[pygame.K_d]:
        pos.x += speed * dt

    pos.x = max(radius, min(pos.x, width - radius))
    pos.y = max(radius, min(pos.y, height - radius))

    # flip() the display to put your work on screen
    pygame.display.flip()

    # limits FPS to 60
    # dt is delta time in seconds since last frame, used for framerate-
    # independent physics.
    dt = clock.tick(60) / 1000

pygame.quit()
