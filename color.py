#!/usr/bin/env python3
import json
import sys
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
    gamma = 2.0
    s = s ** gamma

    # Brightness
    # sRGB: rx, gx, bx = 0.2126, 0.7152, 0.0722
    rx, gx, bx = 0.2126, 0.7152, 0.0722
    v = (rx * r + gx * g + bx * b)

    return int(h), int(s*255), int(v*255)


with open('colors.json', 'r') as f:
    colors = json.loads(f.read())

hsv_colors = []
for label, r, g, b in colors:
    h, s, v = rgb_to_hsv(r, g, b)
    hsv_colors.append((label, (h, s, v)))
hsv_colors.sort(key=itemgetter(1), reverse=True)


# page: 0 or 1
# colors: an array of 127 colors.
# With colors=None, this should set the launchpad to match to the
# programmer's reference, page 10.
def draw(page, colors):
    i = page * 64
    for row in range(8):
        for col in range(8):
            pos = 10 * (8-row) + (1+col)
            if colors is not None:
                try:
                    color = colors[i][0]
                except IndexError:
                    color = '00'
            else:
                color = f'{i:02x}'
            if col > 0:
                print(' ', end='')
            print(color, end='')
            pos = f'{pos:02x}'
            subprocess.run(["./scripts/setcolor", pos, color], check=True)
            i += 1
        print('')


def dump_hsv():
    for i in sorted(hsv_colors, key=itemgetter(0)):
        print(i)


def dump_rgb():
    for i in colors:
        (label, r, g, b) = i
        print(f'{label} #{r:02x}{g:02x}{b:02x}')


# Use this to look at colors in arbitrary groupings.

manual_colors = [
    # red
    '48', '05', '78', '6A', '06', '07', '79', '00',
    # red/orange
    '3C', '00', '00', '00', '00', '00', '00', '00',
    # orange
    '54', '09', '0A', '0B', '7F', '00', '00', '00',
    # orange/hellow
    '7E', '3E', '00', '00', '00', '00', '00', '00',
    # yellow
    '0D', '0E', '0F', '7D', '7C', '3F', '00', '00',
    # green
    '15', '16', '17', '7B', '77', '03', '02', '01',
    # blue
    '4F', '29', '2A', '2B', '00', '00', '00', '67',
    # magenta
    '34', '35', '36', '47', '00', '00', '00', '00',
    # purple
    '31', '50', '45', '32', '00', '00', '00', '70',
    # cyan
    '4D', '21', '1E', '1F', '00', '00', '00', '00',
    # scale colors: off
    '2D', '35', '06', '25', '01', '00', '29', '5E',
    # scale colors: on
    '15', '38', '09', '0D', '03', '00', '34', '53',
]

manual_colors = list([i] for i in manual_colors)

to_draw = None
try:
    page = int(sys.argv[1])
    which = sys.argv[2]
    if which == 'manual':
        to_draw = manual_colors
    elif which == 'hsv':
        to_draw = hsv_colors
    else:
        to_draw = None
except Exception:
    exit('Usage: color.py {0|1} {manual|hsv|none}')


draw(page, to_draw)
# dump_hsv()
# dump_rgb()
