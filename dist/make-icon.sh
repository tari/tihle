#!/bin/sh

for size in 16 32 48 256
do
    magick convert -background none tihle.svg -resize ${size}x${size} ${size}.png
    pngquant ${size}.png
    optipng -o9 ${size}-fs8.png
done

magick convert {16,32,48,256}-fs8.png tihle.ico
