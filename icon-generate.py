import struct, zlib, os, shutil

# -----------------------------------------------------------------------
# 案A デザイン: キーキャップ + チェックマーク
# -----------------------------------------------------------------------
def make_icon_png(size):
    img = [[(0,0,0,0)]*size for _ in range(size)]

    def set_pixel(x, y, r, g, b, a=255):
        if 0 <= x < size and 0 <= y < size:
            ar, ag, ab, aa = img[y][x]
            fa = a / 255.0
            ba = aa / 255.0
            out_a = fa + ba * (1 - fa)
            if out_a > 0:
                out_r = int((r * fa + ar * ba * (1 - fa)) / out_a)
                out_g = int((g * fa + ag * ba * (1 - fa)) / out_a)
                out_b = int((b * fa + ab * ba * (1 - fa)) / out_a)
                img[y][x] = (out_r, out_g, out_b, int(out_a * 255))

    def fill_rounded_rect(x0, y0, x1, y1, r, color, alpha=255):
        rr, gg, bb = color
        for y in range(y0, y1+1):
            for x in range(x0, x1+1):
                in_rect = True
                if x < x0+r and y < y0+r:
                    if (x-x0-r)**2 + (y-y0-r)**2 > r*r: in_rect = False
                elif x > x1-r and y < y0+r:
                    if (x-x1+r)**2 + (y-y0-r)**2 > r*r: in_rect = False
                elif x < x0+r and y > y1-r:
                    if (x-x0-r)**2 + (y-y1+r)**2 > r*r: in_rect = False
                elif x > x1-r and y > y1-r:
                    if (x-x1+r)**2 + (y-y1+r)**2 > r*r: in_rect = False
                if in_rect:
                    set_pixel(x, y, rr, gg, bb, alpha)

    def fill_circle(cx, cy, radius, color, alpha=255):
        rr, gg, bb = color
        for y in range(int(cy-radius)-1, int(cy+radius)+2):
            for x in range(int(cx-radius)-1, int(cx+radius)+2):
                if (x-cx)**2 + (y-cy)**2 <= radius**2:
                    set_pixel(x, y, rr, gg, bb, alpha)

    def draw_letter_K(cx, cy, height, color):
        rr, gg, bb = color
        stroke = max(2, height // 10)
        h = height
        for dy in range(-h//2, h//2):
            for dx in range(-stroke, stroke):
                set_pixel(int(cx - h//4 + dx), int(cy + dy), rr, gg, bb)
        steps = h // 2
        for i in range(steps):
            t = i / steps
            x = cx - h//4 + stroke + int(t * (h//2))
            y = cy - int(t * h//2)
            for dx in range(-stroke, stroke):
                for dy in range(-stroke, stroke):
                    set_pixel(x+dx, y+dy, rr, gg, bb)
        for i in range(steps):
            t = i / steps
            x = cx - h//4 + stroke + int(t * (h//2))
            y = cy + int(t * h//2)
            for dx in range(-stroke, stroke):
                for dy in range(-stroke, stroke):
                    set_pixel(x+dx, y+dy, rr, gg, bb)

    def draw_checkmark(cx, cy, size_c, color):
        rr, gg, bb = color
        stroke = max(1, size_c // 8)
        p1 = (cx - size_c*0.35, cy + size_c*0.05)
        p2 = (cx - size_c*0.05, cy + size_c*0.35)
        p3 = (cx + size_c*0.40, cy - size_c*0.30)
        def draw_line(ax, ay, bx, by):
            steps = max(int(max(abs(bx-ax), abs(by-ay))) * 3, 1)
            for i in range(steps+1):
                t = i/steps
                x = ax + t*(bx-ax)
                y = ay + t*(by-ay)
                for dx in range(-stroke, stroke+1):
                    for dy in range(-stroke, stroke+1):
                        if dx*dx+dy*dy <= stroke*stroke:
                            set_pixel(int(x+dx), int(y+dy), rr, gg, bb)
        draw_line(p1[0], p1[1], p2[0], p2[1])
        draw_line(p2[0], p2[1], p3[0], p3[1])

    s = size
    margin = s * 0.04
    r_outer = int(s * 0.20)
    fill_rounded_rect(int(margin), int(margin),
                      int(s-margin-1), int(s-margin-1),
                      r_outer, (0x18, 0x18, 0x1c))
    pad1 = s * 0.10
    pad_b = s * 0.16
    r_inner = int(s * 0.11)
    fill_rounded_rect(int(margin+pad1), int(margin+pad1*0.6),
                      int(s-margin-pad1-1), int(s-margin-pad_b-1),
                      r_inner, (0x22, 0x22, 0x2a))
    draw_letter_K(int(s*0.44), int(s*0.44), int(s*0.44), (0x6e, 0xe7, 0xf7))
    badge_cx = int(s * 0.75)
    badge_cy = int(s * 0.75)
    badge_r  = int(s * 0.20)
    fill_circle(badge_cx, badge_cy, badge_r + int(s*0.03), (0x0d, 0x0d, 0x0f))
    fill_circle(badge_cx, badge_cy, badge_r, (0x22, 0xc5, 0x5e))
    draw_checkmark(badge_cx, badge_cy, badge_r, (0xff, 0xff, 0xff))

    # PNG エンコード (RGBA)
    def chunk(name, data):
        c = zlib.crc32(name + data) & 0xFFFFFFFF
        return struct.pack('>I', len(data)) + name + data + struct.pack('>I', c)
    sig  = b'\x89PNG\r\n\x1a\n'
    ihdr = chunk(b'IHDR', struct.pack('>IIBBBBB', size, size, 8, 6, 0, 0, 0))
    raw  = b''
    for row in img:
        raw += b'\x00'
        for r,g,b,a in row:
            raw += bytes([r,g,b,a])
    idat = chunk(b'IDAT', zlib.compress(raw, 9))
    iend = chunk(b'IEND', b'')
    return sig + ihdr + idat + iend

# -----------------------------------------------------------------------
# 正しいICO形式で生成
# -----------------------------------------------------------------------
def make_ico(sizes):
    images = [(sz, make_icon_png(sz)) for sz in sizes]
    count  = len(images)
    header = struct.pack('<HHH', 0, 1, count)
    dir_size    = count * 16
    offset      = 6 + dir_size
    entries = b''
    data    = b''
    for sz, png in images:
        w = sz if sz < 256 else 0
        h = sz if sz < 256 else 0
        entries += struct.pack('<BBBBHHII', w, h, 0, 0, 1, 32, len(png), offset)
        data    += png
        offset  += len(png)
    return header + entries + data

# -----------------------------------------------------------------------
# ファイル出力
# -----------------------------------------------------------------------
os.makedirs("src-tauri/icons", exist_ok=True)

for name, sz in [("32x32", 32), ("128x128", 128), ("128x128@2x", 256)]:
    data = make_icon_png(sz)
    with open(f"src-tauri/icons/{name}.png", "wb") as f:
        f.write(data)
    print(f"Generated {name}.png")

ico_data = make_ico([16, 32, 48, 256])
with open("src-tauri/icons/icon.ico", "wb") as f:
    f.write(ico_data)
print(f"Generated icon.ico ({len(ico_data)} bytes)")

shutil.copy("src-tauri/icons/128x128.png", "src-tauri/icons/icon.icns")
print("Done")