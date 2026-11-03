import pathlib
import struct
import sys
import zlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
SCALE = 16
ART = [
    "................",
    "..HHHHHHHHHHHH..",
    ".HhhhhhhhhhhhhH.",
    ".HhKKKKKKKKKKhHd",
    ".HhKSSSSSSSSKhHd",
    ".HhKSSSSSSSSKhHd",
    ".HhKSWBSSWBSKhHd",
    ".HhKSSSSSSSSKhHd",
    ".HhKSSNNNNSSKhHd",
    ".HhKSSSSSSSSKhHd",
    ".HhKKKKKKKKKKhHd",
    ".HhhhhhhhhhhhhHd",
    "..HHHHHHHHHHHHdd",
    "...dddddddddddd.",
    "................",
    "................",
]
COLORS = {
    "H": (58, 120, 214, 255),
    "h": (96, 160, 240, 255),
    "d": (24, 40, 80, 255),
    "K": (40, 28, 20, 255),
    "S": (232, 178, 130, 255),
    "W": (250, 250, 250, 255),
    "B": (60, 90, 200, 255),
    "N": (150, 92, 70, 255),
    ".": (0, 0, 0, 0),
}


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def png(art, scale):
    size = len(art) * scale
    rows = bytearray()
    for line in art:
        assert len(line) == len(art), line
        row = b"".join(bytes(COLORS[c]) * scale for c in line)
        rows += (b"\x00" + row) * scale
    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b"")


def main():
    target = ROOT / "icon.png"
    target.write_bytes(png(ART, SCALE))
    print(target, target.stat().st_size, "bytes")


if __name__ == "__main__":
    sys.exit(main())
