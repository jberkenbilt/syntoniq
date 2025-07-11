#!/usr/bin/env python3
import json
from operator import itemgetter
import subprocess


def rgb_to_hsv(r, g, b):
    r, g, b = r / 255, g / 255, b / 255
    mx = max(r, g, b)
    mn = min(r, g, b)
    diff = mx - mn

    if diff == 0:
        h = 0
    elif mx == r:
        h = (60 * ((g - b) / diff) + 360) % 360
    elif mx == g:
        h = (60 * ((b - r) / diff) + 120) % 360
    else:  # mx == b
        h = (60 * ((r - g) / diff) + 240) % 360

    # Saturation
    s = 0 if mx == 0 else (diff / mx)
    gamma = 1.0
    s = s ** gamma

    # Brightness
    # sRGB: rx, gx, bx = 0.2126, 0.7152, 0.0722
    rx, gx, bx = 0.2126, 0.7152, 0.0722
    v = (rx * r + gx * g + bx * b)

    return h, s, v


with open('colors.json', 'r') as f:
    colors = json.loads(f.read())

hsv_colors = []
for label, r, g, b in colors:
    h, s, v = rgb_to_hsv(r, g, b)
    hsv_colors.append((label, (h, s, v)))
hsv_colors.sort(key=itemgetter(1), reverse=True)

i = 0
for row in range(8):
    for col in range(8):
        pos = 10 * (1+row) + (1+col)
        color = hsv_colors[i][0]
        print(pos, color)
        pos = f'{pos:02x}'
        subprocess.run(["./scripts/setcolor", pos, color], check=True)
        i += 1
