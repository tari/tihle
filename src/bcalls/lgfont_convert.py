#!/usr/bin/env python
#
# Convert lgfont.png, a 256x1-character image to a character-major blob of
# bytes, each character 7 bytes.

from PIL import Image

im = list(Image.open('lgfont.png').getdata())

out = bytearray()
for c in range(256):
    for row in range(7):
        x = 0
        for bit in range(8):
            # 255 is white, 0 is black; convert to 1 bpp 1=black
            pixel = int(not bool(im[2048 * row + 8 * c + bit]))
            x = (x << 1) | pixel
        out.append(x)

open('lgfont.bin', 'wb').write(out)
