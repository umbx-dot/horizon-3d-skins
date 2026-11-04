import pathlib
import struct
import sys
import zlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
GRID = 32
SCALE = 8
TEXEL = 2
HEAD_X, HEAD_Y, HEAD_SIZE, DEPTH = 5, 10, 16, 4

CLEAR = (0, 0, 0, 0)
RIM = (58, 66, 112, 255)
PANEL = (27, 29, 54, 255)
PANEL_DEEP = (22, 23, 44, 255)
BRACKET = (196, 206, 240, 255)
GAP = (16, 13, 20, 255)

TEXELS = {
    "h": (104, 68, 42),
    "H": (134, 92, 58),
    "s": (230, 174, 128),
    "S": (208, 152, 110),
    "w": (246, 246, 246),
    "e": (60, 98, 216),
    "n": (186, 124, 88),
    "m": (142, 82, 62),
}
FACE = [
    "hhhhhhhh",
    "hhhhhhhh",
    "hsssssSh",
    "ssssssss",
    "swessews",
    "sssnnsss",
    "ssmmmmss",
    "ssssssss",
]
SIDE = [
    "hhhhhhhh",
    "hhhhhhhh",
    "hhhhhhhs",
    "hhhhssss",
    "hhhsSsss",
    "hhssssss",
    "hhssssss",
    "Shssssss",
]
TOP = [
    "hHhhHhhH",
    "HhhHhhHh",
    "hhHhhHhh",
    "hHhhhhHh",
    "HhhHhhhH",
    "hhHhHhhh",
]
FRINGE_FRONT = [5, 4, 5, 3, 3, 4, 3, 4, 5]
FRINGE_SIDE = [5, 6, 5, 6, 7]
LIGHT = {"front": 0.94, "top": 1.06, "right": 0.7}


def lit(key, face):
    return tuple(min(255, round(c * LIGHT[face])) for c in TEXELS[key]) + (255,)


def frame(x, y):
    edge = min(x, y, GRID - 1 - x, GRID - 1 - y)
    near_x, near_y = min(x, GRID - 1 - x), min(y, GRID - 1 - y)
    if near_x + near_y < 2:
        return CLEAR
    if edge < 2 or near_x + near_y < 3:
        return RIM
    if (near_x == 3 and 3 <= near_y <= 6) or (near_y == 3 and 3 <= near_x <= 6):
        return BRACKET
    return PANEL_DEEP if y > GRID - 9 else PANEL


def draw_head(grid):
    for k in range(1, DEPTH + 1):
        x = HEAD_X + HEAD_SIZE - 1 + k
        for row in range(HEAD_SIZE):
            grid[HEAD_Y + row - k][x] = lit(SIDE[row // TEXEL][(k - 1) * 8 // DEPTH], "right")
        for column in range(HEAD_SIZE):
            grid[HEAD_Y - k][HEAD_X + column + k] = lit(TOP[(DEPTH - k) * 6 // DEPTH][column // TEXEL], "top")
    for row in range(HEAD_SIZE):
        for column in range(HEAD_SIZE):
            grid[HEAD_Y + row][HEAD_X + column] = lit(FACE[row // TEXEL][column // TEXEL], "front")


def draw_shell(grid):
    x0, y0, size, depth = HEAD_X - 1, HEAD_Y - 1, HEAD_SIZE + 2, DEPTH + 1
    for k in range(1, depth + 1):
        x = x0 + size - 1 + k
        reach = FRINGE_SIDE[k - 1]
        for row in range(reach):
            grid[y0 + row - k][x] = lit("H" if row == 0 else "h", "right")
        grid[y0 + reach - k][x] = GAP
        for column in range(size):
            grid[y0 - k][x0 + column + k] = lit(TOP[(depth - k) * 6 // depth][column * 8 // size], "top")
    for column in range(size):
        reach = FRINGE_FRONT[column // TEXEL]
        for row in range(reach):
            grid[y0 + row][x0 + column] = lit("H" if row == 0 else "h", "front")
        if 0 < column < size - 1:
            grid[y0 + reach][x0 + column] = GAP


def main():
    grid = [[frame(x, y) for x in range(GRID)] for y in range(GRID)]
    draw_head(grid)
    draw_shell(grid)
    assert all(len(line) == GRID and all(len(c) == 4 for c in line) for line in grid)
    size = GRID * SCALE
    rows = bytearray()
    for line in grid:
        rows += (b"\x00" + b"".join(bytes(color) * SCALE for color in line)) * SCALE
    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    target = ROOT / "icon.png"
    target.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b""))
    print(target, target.stat().st_size, "bytes")


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


if __name__ == "__main__":
    sys.exit(main())
