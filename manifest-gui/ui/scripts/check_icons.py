"""Verify the generated icon set exists and is not the gray placeholder.
Run from manifest-gui/ui: python3 scripts/check_icons.py"""
import os
import sys
from PIL import Image

ICONS = os.path.join(os.path.dirname(__file__), "..", "..", "icons")
REQUIRED = ["32x32.png", "128x128.png", "128x128@2x.png", "icon.icns", "icon.ico", "icon.png"]

def main():
    missing = [n for n in REQUIRED if not os.path.exists(os.path.join(ICONS, n))]
    if missing:
        print(f"missing: {missing}")
        return 1
    img = Image.open(os.path.join(ICONS, "icon.png")).convert("RGBA")
    if img.size[0] < 512:
        print(f"icon.png too small: {img.size}")
        return 1
    colors = img.getcolors(maxcolors=4096)
    if colors is not None and len(colors) < 16:
        print(f"icon.png looks like a flat placeholder ({0 if colors is None else len(colors)} colors)")
        return 1
    print("icon set ok")
    return 0

if __name__ == "__main__":
    sys.exit(main())
