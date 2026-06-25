#!/usr/bin/env python3
"""Generate the Reveille app icon set: a rising sun over a green horizon (reveille = dawn)."""
import os
from PIL import Image, ImageDraw

S = 1024
# Repo-relative: scripts/ -> ../src-tauri/icons
OUT = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(OUT, exist_ok=True)

def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))

img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(img)

horizon = int(S * 0.66)

# --- Sky: deep dawn-blue at top fading to warm gold at the horizon ---
sky_top = (18, 32, 64)      # deep indigo
sky_low = (255, 170, 71)    # warm gold
for y in range(horizon):
    t = y / horizon
    # ease so most of the warm band sits near the horizon
    t = t ** 1.6
    d.line([(0, y), (S, y)], fill=lerp(sky_top, sky_low, t))

# --- Sun rays (drawn before the sun so the disc sits on top) ---
cx, cy = S // 2, horizon
ray = (255, 224, 130, 120)
import math
for k in range(12):
    ang = math.radians(180 + (k + 0.5) * (180 / 12))  # upper half only
    x1 = cx + math.cos(ang) * 250
    y1 = cy + math.sin(ang) * 250
    x2 = cx + math.cos(ang) * 470
    y2 = cy + math.sin(ang) * 470
    d.line([(x1, y1), (x2, y2)], fill=ray, width=14)

# --- Sun disc (bright gold) ---
r = 215
d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(255, 209, 74))
# inner glow
d.ellipse([cx - r + 40, cy - r + 40, cx + r - 40, cy + r - 40], fill=(255, 233, 150))

# --- Ground: military green, gradient, covers the lower half of the sun ---
g_top = (74, 140, 63)
g_bot = (32, 64, 36)
for y in range(horizon, S):
    t = (y - horizon) / (S - horizon)
    d.line([(0, y), (S, y)], fill=lerp(g_top, g_bot, t))

# thin bright horizon line
d.line([(0, horizon), (S, horizon)], fill=(180, 230, 150), width=6)

# --- Rounded-square mask (app-icon shape) ---
mask = Image.new("L", (S, S), 0)
ImageDraw.Draw(mask).rounded_rectangle([0, 0, S - 1, S - 1], radius=185, fill=255)
img.putalpha(mask)

# --- Export the sizes Tauri references ---
img.save(os.path.join(OUT, "icon.png"))
for name, sz in [("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256)]:
    img.resize((sz, sz), Image.LANCZOS).save(os.path.join(OUT, name))

# Windows .ico (multi-resolution) and macOS .icns for completeness
ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
img.save(os.path.join(OUT, "icon.ico"), sizes=ico_sizes)
try:
    img.resize((1024, 1024), Image.LANCZOS).save(os.path.join(OUT, "icon.icns"))
except Exception as e:
    print("icns skip:", e)

print("wrote:", sorted(os.listdir(OUT)))
