#!/usr/bin/env python3
"""Re-pad the Android adaptive-icon foreground layers to Google's safe zone.

`npx tauri icon app-icon-source.png` resizes the source image 1:1 into each
`ic_launcher_foreground.png` canvas, without leaving the safe-zone margin
Android needs to mask the icon into a circle/squircle/rounded-square without
clipping the logo. Re-run this script after any `tauri icon` regeneration.

Usage: python3 scripts/fix_android_adaptive_icon.py
Requires: pip install Pillow
"""

import os

from PIL import Image

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SOURCE = os.path.join(REPO_ROOT, "app-icon-source.png")
ANDROID_ICONS_DIR = os.path.join(REPO_ROOT, "src-tauri/icons/android")

# Google's guideline: keep the logo inside a circle 66dp wide in the 108dp
# adaptive-icon canvas (~61%). 60% leaves a little extra margin so OEM
# launcher masks that are slightly tighter than stock Android don't clip it.
SAFE_FRACTION = 0.60

DENSITY_CANVAS_SIZES = {
    "mdpi": 108,
    "hdpi": 162,
    "xhdpi": 216,
    "xxhdpi": 324,
    "xxxhdpi": 432,
}


def main():
    source = Image.open(SOURCE).convert("RGBA")
    logo = source.crop(source.split()[-1].getbbox())
    logo_w, logo_h = logo.size

    for density, canvas_size in DENSITY_CANVAS_SIZES.items():
        scale = (canvas_size * SAFE_FRACTION) / max(logo_w, logo_h)
        new_size = (max(1, round(logo_w * scale)), max(1, round(logo_h * scale)))
        resized = logo.resize(new_size, Image.LANCZOS)

        canvas = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
        paste_pos = ((canvas_size - new_size[0]) // 2, (canvas_size - new_size[1]) // 2)
        canvas.paste(resized, paste_pos, resized)

        out_path = os.path.join(ANDROID_ICONS_DIR, f"mipmap-{density}", "ic_launcher_foreground.png")
        canvas.save(out_path)
        print(f"{density}: canvas={canvas_size}px logo={new_size[0]}x{new_size[1]}px -> {out_path}")


if __name__ == "__main__":
    main()
