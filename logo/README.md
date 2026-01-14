# Logo

The logo was hand-coded in PostScript. The script `./make-svg` was used as a one-shot create the SVG.

The logo was manually copied to ../manual/static/ and to the static top-level syntoniq.cc website.

To create PNG where the logo body is $size pixels high and is padded
enough to fit in a circle:

```sh
size=128
ext=$(python3 -c "import math; print(math.ceil($size * 1.2))")
convert -background none syntoniq-logo.svg -resize ${size}x${size} \
    -gravity center -extent ${ext}x${ext} \
    syntoniq-logo-$size.png
```
