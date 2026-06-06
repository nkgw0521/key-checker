import os
import struct
import zlib

os.makedirs("src-tauri/icons", exist_ok=True)


def make_rgba_png(size):
    """単色(#6ee7f7)のRGBA PNGを生成"""

    def chunk(name, data):
        c = zlib.crc32(name + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + name + data + struct.pack(">I", c)

    sig = b"\x89PNG\r\n\x1a\n"
    # color type=6 がRGBA
    ihdr = chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
    # RGBA: R=110, G=231, B=247, A=255
    raw = b"".join(b"\x00" + bytes([110, 231, 247, 255] * size) for _ in range(size))
    idat = chunk(b"IDAT", zlib.compress(raw))
    iend = chunk(b"IEND", b"")
    return sig + ihdr + idat + iend


for name, sz in [("32x32", 32), ("128x128", 128), ("128x128@2x", 256)]:
    path = f"src-tauri/icons/{name}.png"
    with open(path, "wb") as f:
        f.write(make_rgba_png(sz))
    print(f"Generated: {path}")

import shutil

shutil.copy("src-tauri/icons/128x128.png", "src-tauri/icons/icon.icns")
shutil.copy("src-tauri/icons/32x32.png", "src-tauri/icons/icon.ico")
print("Done")
