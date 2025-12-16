#!/usr/bin/env python3
#
# This simple script helps you tell how close various EDO scale steps
# are to 12-tone semitones, which can help with picking note names.
#
import sys
import os
import math

whoami = os.path.basename(sys.argv[0])

try:
    edo = int(sys.argv[1])
except Exception:
    exit(f'usage {whoami} EDO')

print("""
 0  C
 1  C#
 2  D
 3  E%
 4  E
 5  F
 6  F#
 7  G
 8  A%
 9  A
10  B%
11  B
""")

for i in range(edo+1):
    cents = (2 ** (i/edo))
    semitones = math.log2(cents) * 12
    print(f'{i:2} ', round(semitones * 1000)/1000)
