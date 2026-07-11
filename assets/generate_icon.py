"""Generate keisen app icons: dark gray keycap with white 田."""
from PIL import Image, ImageDraw
from pathlib import Path
import struct
import io

OUT = Path(__file__).resolve().parent
SIZES = [16, 24, 32, 48, 64, 128, 256]


def make_keycap(size: int) -> Image.Image:
    scale = 4 if size <= 64 else 2
    S = size * scale
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    m = max(scale, round(S * 0.07))
    r = max(scale * 2, round(S * 0.16))

    d.rounded_rectangle(
        [m + scale, m + round(S * 0.09), S - m - 1, S - m - 1],
        radius=r,
        fill=(16, 18, 20, 210),
    )
    d.rounded_rectangle(
        [m, m + round(S * 0.035), S - m - 1, S - m - 1 - round(S * 0.02)],
        radius=r,
        fill=(40, 44, 50, 255),
    )
    fi = max(scale, round(S * 0.11))
    face = [
        m + fi,
        m + fi + round(S * 0.02),
        S - m - fi - 1,
        S - m - fi - 1 - round(S * 0.07),
    ]
    d.rounded_rectangle(face, radius=max(scale * 2, r - scale * 2), fill=(52, 56, 62, 255))

    sheen_h = max(scale, round(S * 0.10))
    for i in range(sheen_h):
        a = int(40 * (1 - i / max(1, sheen_h)))
        y = face[1] + scale + i
        if y >= face[3] - scale * 2:
            break
        d.line(
            [(face[0] + scale * 2, y), (face[2] - scale * 2, y)],
            fill=(88, 94, 102, a),
        )

    pad = max(scale * 2, round((face[2] - face[0]) * 0.22))
    x0 = face[0] + pad
    y0 = face[1] + pad
    x1 = face[2] - pad
    y1 = face[3] - pad
    gw, gh = x1 - x0, y1 - y0
    t = max(scale * 2, round(min(gw, gh) * 0.10))
    while t * 3 >= min(gw, gh) and t > scale:
        t -= 1

    white = (250, 252, 255, 255)

    def hbar(y_center):
        y = int(y_center - t / 2)
        d.rectangle([x0, y, x1, y + t - 1], fill=white)

    def vbar(x_center):
        x = int(x_center - t / 2)
        d.rectangle([x, y0, x + t - 1, y1], fill=white)

    hbar(y0 + t / 2)
    hbar(y1 - t / 2 + 0.5)
    vbar(x0 + t / 2)
    vbar(x1 - t / 2 + 0.5)
    hbar((y0 + y1) / 2)
    vbar((x0 + x1) / 2)

    if scale != 1:
        img = img.resize((size, size), Image.Resampling.LANCZOS)
    return img


def write_ico(path: Path, images: list[Image.Image]):
    entries, payloads = [], []
    offset = 6 + 16 * len(images)
    for im in images:
        buf = io.BytesIO()
        im.save(buf, format="PNG")
        data = buf.getvalue()
        bw = 0 if im.width >= 256 else im.width
        bh = 0 if im.height >= 256 else im.height
        entries.append(struct.pack("<BBBBHHII", bw, bh, 0, 0, 1, 32, len(data), offset))
        payloads.append(data)
        offset += len(data)
    with open(path, "wb") as f:
        f.write(struct.pack("<HHH", 0, 1, len(images)))
        for e in entries:
            f.write(e)
        for p in payloads:
            f.write(p)


def main():
    images = []
    for s in SIZES:
        im = make_keycap(s)
        images.append(im)
        im.save(OUT / f"icon-{s}.png")
    images[-1].save(OUT / "icon.png")
    write_ico(OUT / "icon.ico", images)
    print("wrote", OUT / "icon.png", OUT / "icon.ico")


if __name__ == "__main__":
    main()
