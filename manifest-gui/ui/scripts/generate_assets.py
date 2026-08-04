"""Deterministic pixel assets for manifest-gui. Run from manifest-gui/ui:
python3 scripts/generate_assets.py
Requires Pillow (pip3 install --user pillow). Outputs are committed;
this script only runs when regenerating them. Robert can replace any
output with hand-made art; keep filenames stable."""
import random
from PIL import Image

INK = (43, 38, 34)
SEAL_DARK = (128, 32 , 24)
SEAL_MID = (160, 48, 36)
SEAL_HI = (196, 84, 64)
RED = (160, 48, 36)

def paper(path, size=128, seed=7):
    rng = random.Random(seed)
    base = (236, 229, 211)
    img = Image.new("RGB", (size, size), base)
    px = img.load()
    for y in range(size):
        for x in range(size):
            n = rng.randint(-6, 6)
            if rng.random() < 0.035:
                n -= rng.randint(4, 12)
            r, g, b = base
            px[x, y] = (r + n, g + n, b + n - (1 if n < 0 else 0))
    img.save(path)

SEAL_ART = [
    "........####........",
    "......########......",
    ".....##########.....",
    "....####MMMM####....",
    "...###MMMMMMMM###...",
    "...##MMHHHHHHMM##...",
    "..###MHHMMMMHHM###..",
    "..##MMHMM##MMHMM##..",
    "..##MMHM####MHMM##..",
    "..##MMHM####MHMM##..",
    "..##MMHMM##MMHMM##..",
    "..###MHHMMMMHHM###..",
    "...##MMHHHHHHMM##...",
    "...###MMMMMMMM###...",
    "....####MMMM####....",
    ".....##########.....",
    "......########......",
    "........####........",
]

GLYPH_ART = [
    "................",
    "......###.......",
    ".....#####......",
    ".....##.##......",
    "....##...##.....",
    "....##.#.##.....",
    "...##..#..##....",
    "...##..#..##....",
    "..##...#...##...",
    "..##.......##...",
    ".##....#....##..",
    ".##...........#.",
    "##############..",
    "................",
    "................",
    "................",
]

def art(path, rows, colors):
    h = len(rows)
    w = max(len(r) for r in rows)
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    px = img.load()
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch in colors:
                px[x, y] = colors[ch] + (255,)
    img.save(path)

paper("src/assets/paper.png")
art("src/assets/seal.png", SEAL_ART, {"#": SEAL_DARK, "M": SEAL_MID, "H": SEAL_HI})
art("src/assets/glyph-error.png", GLYPH_ART, {"#": RED})
print("wrote paper.png, seal.png, glyph-error.png")
