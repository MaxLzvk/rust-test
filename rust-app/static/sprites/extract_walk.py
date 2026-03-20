from PIL import Image
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

def make_strip(src, frames, out_name, frame_size=32):
    img = Image.open(src).convert("RGBA")
    n = len(frames)
    strip = Image.new("RGBA", (frame_size * n, frame_size), (0, 0, 0, 0))
    for i, box in enumerate(frames):
        frame = img.crop(box)
        frame = frame.resize((frame_size, frame_size), Image.NEAREST)
        strip.paste(frame, (i * frame_size, 0), frame)
    strip.save(out_name)
    print(f"  {out_name}: {n} frames, {frame_size}x{frame_size}")

print("Extracting walk sprites...")

# Slime (46x52) - top row bounce frames
make_strip("slime.png", [
    (0, 0, 9, 13),
    (9, 0, 18, 13),
    (18, 0, 28, 13),
    (9, 0, 18, 13),
], "slime_walk.png")

# Black Mage (280x295) - walk down row starts y=55
make_strip("mage.png", [
    (0, 55, 40, 95),
    (40, 55, 80, 95),
    (80, 55, 120, 95),
    (40, 55, 80, 95),
], "mage_walk.png")

# Daisy Kart (401x223) - row of karts at y=50-85
# First kart (face down) at ~x=50, second at ~x=105, etc
make_strip("kart.png", [
    (50, 50, 95, 88),
    (105, 50, 150, 88),
    (160, 50, 205, 88),
    (105, 50, 150, 88),
], "kart_walk.png")

# Mega Man Soccer (346x2486) - walk in top rows
make_strip("megaman.png", [
    (0, 0, 40, 46),
    (40, 0, 80, 46),
    (80, 0, 120, 46),
    (40, 0, 80, 46),
], "megaman_walk.png")

# Mermaid (1194x494) - small poses: left side ~(160,80)-(260,200), right ~(1000,80)-(1100,200)
# Central portrait ~(480,20)-(720,200)
make_strip("mermaid.png", [
    (160, 80, 280, 200),
    (480, 20, 720, 200),
    (1000, 60, 1100, 180),
    (480, 20, 720, 200),
], "mermaid_walk.png")

print("Done!")
