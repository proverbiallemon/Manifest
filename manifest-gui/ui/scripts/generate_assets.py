"""Deterministic pixel assets for manifest-gui. Run from manifest-gui/ui:
python3 scripts/generate_assets.py
Requires Pillow (pip3 install --user pillow). Outputs are committed;
this script only runs when regenerating them. Robert can replace any
output with hand-made art; keep filenames stable."""
import os
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

def app_icon(path, size=1024):
    """Pixel-paper app icon: rounded paper card, double ink rule border,
    the red wax seal centered. Drawn on a 64-cell grid, nearest upscaled."""
    grid = 64
    cell = size // grid
    rng = random.Random(11)
    base = (236, 229, 211)
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    px = img.load()
    radius = 7

    def inside(gx, gy):
        rx = min(gx, grid - 1 - gx)
        ry = min(gy, grid - 1 - gy)
        if rx >= radius or ry >= radius:
            return True
        dx, dy = radius - rx, radius - ry
        return dx * dx + dy * dy <= radius * radius

    def ring(gx, gy, d):
        rx = min(gx, grid - 1 - gx)
        ry = min(gy, grid - 1 - gy)
        return min(rx, ry) == d and inside(gx, gy)

    # paper cells with grain
    cells = {}
    for gy in range(grid):
        for gx in range(grid):
            if not inside(gx, gy):
                continue
            n = rng.randint(-6, 6)
            if rng.random() < 0.035:
                n -= rng.randint(4, 12)
            cells[(gx, gy)] = (base[0] + n, base[1] + n, base[2] + n, 255)
    # double ink rule border (grid rings 2 and 4)
    for gy in range(grid):
        for gx in range(grid):
            if ring(gx, gy, 2) or ring(gx, gy, 4):
                cells[(gx, gy)] = (*INK, 255)
    # seal centered, 2 grid cells per art pixel
    art_w = len(SEAL_ART[0])
    art_h = len(SEAL_ART)
    scale = 2
    ox = (grid - art_w * scale) // 2
    oy = (grid - art_h * scale) // 2
    palette = {"#": SEAL_DARK, "M": SEAL_MID, "H": SEAL_HI}
    for ay, row in enumerate(SEAL_ART):
        for ax, ch in enumerate(row):
            color = palette.get(ch)
            if color is None:
                continue
            for sy in range(scale):
                for sx in range(scale):
                    cells[(ox + ax * scale + sx, oy + ay * scale + sy)] = (*color, 255)
    # rasterize grid cells
    for (gx, gy), color in cells.items():
        for yy in range(gy * cell, (gy + 1) * cell):
            for xx in range(gx * cell, (gx + 1) * cell):
                px[xx, yy] = color
    img.save(path)

paper("src/assets/paper.png")
art("src/assets/seal.png", SEAL_ART, {"#": SEAL_DARK, "M": SEAL_MID, "H": SEAL_HI})
art("src/assets/glyph-error.png", GLYPH_ART, {"#": RED})
app_icon(os.path.join(os.path.dirname(__file__), "..", "..", "app-icon.png"))
print("wrote paper.png, seal.png, glyph-error.png")
