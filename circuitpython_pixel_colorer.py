import board
import neopixel

NPX_PIN = board.D6
npx = neopixel.NeoPixel(NPX_PIN, 25, brightness=1., auto_write=True)

while True:
    entry = input()
    if len(entry) != 3:
        print(f'Invalid input "{entry}"')
        continue

    r,g,b = [int(v) for v in entry]

    npx.fill((r,g,b))