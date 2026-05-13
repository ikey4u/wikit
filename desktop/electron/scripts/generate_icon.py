from PIL import Image, ImageDraw, ImageFilter
import math
import os
import struct
import io

SIZE = 1024
PADDING = 96
INNER = SIZE - PADDING * 2
CR = int(220 * INNER / SIZE)


def bg_white():
    img = Image.new('RGBA', (INNER, INNER), (0, 0, 0, 0))
    mask = Image.new('L', (INNER, INNER), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, INNER - 1, INNER - 1), radius=CR, fill=255)
    bg = Image.new('RGBA', (INNER, INNER), (0, 0, 0, 0))
    d = ImageDraw.Draw(bg)
    for y in range(INNER):
        t = y / INNER
        v = min(int(225 + t * 15), 240)
        d.line([(0, y), (INNER, y)], fill=(v, v, v, 255))
    img.paste(bg, mask=mask)
    bd = Image.new('RGBA', (INNER, INNER), (0, 0, 0, 0))
    ImageDraw.Draw(bd).rounded_rectangle((0, 0, INNER - 1, INNER - 1), radius=CR, outline=(180, 183, 190, 120), width=3)
    return Image.alpha_composite(img, bd)


def gradient_ring(draw, cx, cy, r, width, start_color, end_color, start_deg=0, end_deg=360):
    steps = end_deg - start_deg
    for i in range(steps):
        t = i / max(steps - 1, 1)
        color = tuple(int(s + (e - s) * t) for s, e in zip(start_color, end_color))
        angle_start = start_deg + i
        draw.arc((cx - r, cy - r, cx + r, cy + r), start=angle_start, end=angle_start + 2, fill=color, width=width)


def generate_icon():
    scale = INNER / 1024.0
    img = bg_white()
    cx, cy = INNER // 2, INNER // 2 - int(15 * scale)
    r = int(250 * scale)

    shadow = Image.new('RGBA', (INNER, INNER), (0, 0, 0, 0))
    sd = ImageDraw.Draw(shadow)
    sd.ellipse((cx - r - int(10 * scale), cy - r + int(10 * scale),
                cx + r + int(10 * scale), cy + r + int(30 * scale)), fill=(0, 0, 0, 12))
    shadow = shadow.filter(ImageFilter.GaussianBlur(int(20 * scale)))
    img = Image.alpha_composite(img, shadow)

    s = Image.new('RGBA', (INNER, INNER), (0, 0, 0, 0))
    d = ImageDraw.Draw(s)

    gradient_ring(d, cx, cy, r, int(48 * scale), (60, 180, 220), (130, 80, 210), 20, 340)

    ha = math.radians(325)
    hx = cx + r * math.cos(ha)
    hy = cy + r * math.sin(ha)
    handle_len = int(125 * scale)
    ex = hx + handle_len * math.cos(ha)
    ey = hy + handle_len * math.sin(ha)
    dot_r = int(24 * scale)
    for i in range(50):
        t = i / 49
        px = hx + (ex - hx) * t
        py = hy + (ey - hy) * t
        color = tuple(int(s_c + (e_c - s_c) * t) for s_c, e_c in zip((130, 80, 210), (160, 60, 180)))
        d.ellipse((px - dot_r, py - dot_r, px + dot_r, py + dot_r), fill=color)

    sz = int(130 * scale)
    pts = [
        (cx - sz, cy - sz * 0.6),
        (cx - sz * 0.5, cy + sz * 0.6),
        (cx, cy - sz * 0.1),
        (cx + sz * 0.5, cy + sz * 0.6),
        (cx + sz, cy - sz * 0.6),
    ]
    ww = int(34 * scale)
    for i in range(len(pts) - 1):
        num = 30
        for j in range(num):
            t2 = j / (num - 1)
            px = pts[i][0] + (pts[i + 1][0] - pts[i][0]) * t2
            py = pts[i][1] + (pts[i + 1][1] - pts[i][1]) * t2
            seg_t = (i + t2) / (len(pts) - 1)
            cr_c = int(50 + seg_t * 60)
            cg_c = int(150 + seg_t * (-70))
            cb_c = int(200 + seg_t * 10)
            d.ellipse((px - ww // 2, py - ww // 2, px + ww // 2, py + ww // 2), fill=(cr_c, cg_c, min(cb_c, 255)))

    inner_result = Image.alpha_composite(img, s)

    final = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    final.paste(inner_result, (PADDING, PADDING))
    return final


def create_ico(img, path):
    sizes = [16, 24, 32, 48, 64, 128, 256]
    imgs = []
    for sz in sizes:
        resized = img.resize((sz, sz), Image.LANCZOS)
        imgs.append(resized)
    imgs[0].save(path, format='ICO', sizes=[(sz, sz) for sz in sizes], append_images=imgs[1:])


def create_icns(img, path):
    icon_types = [
        (b'ic07', 128),
        (b'ic08', 256),
        (b'ic09', 512),
        (b'ic10', 1024),
        (b'ic11', 32),
        (b'ic12', 64),
        (b'ic13', 256),
        (b'ic14', 512),
    ]

    entries = []
    for icon_type, size in icon_types:
        resized = img.resize((size, size), Image.LANCZOS)
        buf = io.BytesIO()
        resized.save(buf, format='PNG')
        png_data = buf.getvalue()
        entry_data = icon_type + struct.pack('>I', len(png_data) + 8) + png_data
        entries.append(entry_data)

    body = b''.join(entries)
    total_size = 8 + len(body)
    icns_data = b'icns' + struct.pack('>I', total_size) + body

    with open(path, 'wb') as f:
        f.write(icns_data)


def main():
    icon_dir = os.path.join(os.path.dirname(__file__), '..', 'icons')
    os.makedirs(icon_dir, exist_ok=True)

    full = generate_icon()

    full.save(os.path.join(icon_dir, 'icon.png'), 'PNG')
    print("Saved icon.png (1024x1024)")

    img_128 = full.resize((128, 128), Image.LANCZOS)
    img_128.save(os.path.join(icon_dir, '128x128.png'), 'PNG')
    print("Saved 128x128.png")

    img_256 = full.resize((256, 256), Image.LANCZOS)
    img_256.save(os.path.join(icon_dir, '128x128@2x.png'), 'PNG')
    print("Saved 128x128@2x.png")

    create_ico(full, os.path.join(icon_dir, 'icon.ico'))
    print("Saved icon.ico")

    create_icns(full, os.path.join(icon_dir, 'icon.icns'))
    print("Saved icon.icns")

    print("\nAll icon files generated successfully!")


if __name__ == '__main__':
    main()
