"""Builds the complete Brickstrap logo set: SVGs, PNG exports with provenance
metadata, and the Windows .ico.

Mark: a code sign </> whose slash is three round studs climbing like a signal
meter (gray -> silver -> white). Run from the project root: python3 build_logo.py
"""
import subprocess
from pathlib import Path

import uharfbuzz as hb
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont
from PIL import Image, PngImagePlugin

BLACK = "#0B0B0D"
GRAY = "#8B9099"
SILVER = "#C3C7CD"
WHITE = "#FFFFFF"
STUD_INKS = [GRAY, SILVER, WHITE]  # weak -> strong signal, bottom-left to top-right

OUT = Path("logo")
PNG = OUT / "png"
FONT = "vendor/archivo-850-118.ttf"
CONTRACT = ".impeccable/surfaces/logo-brickstrap-icon-svg.md"
OUT.mkdir(exist_ok=True)
PNG.mkdir(exist_ok=True)

# ---- Mark geometry on a 64-unit canvas --------------------------------------
# Two chevrons (round caps/joins) and three studs forming the slash.
# Chevrons: round caps/joins. Studs: flat bottom, round top (the brick stud
# profile), stepping up the diagonal AND growing taller, so the rise survives
# in one ink. About 2.5-3 units clear of the chevron caps.
# Plain code sign </>: silver chevrons, white slash, round caps.
GEOM = {"regular": dict(stroke=6, left=[(22, 21), (10, 32), (22, 43)], right=[(42, 21), (54, 32), (42, 43)],
                        slash=[(28.5, 45), (35.5, 19)])}


def poly(pts):
    return "M" + "L".join(f"{x} {y}" for x, y in pts)


def mark_svg_inner(weight="regular", ink=None, bracket=SILVER):
    g = GEOM[weight]
    b = ink or bracket
    return (f'<g fill="none" stroke-width="{g["stroke"]}" stroke-linecap="round" stroke-linejoin="round">'
            f'<path stroke="{b}" d="{poly(g["left"])}"/><path stroke="{b}" d="{poly(g["right"])}"/>'
            f'<path stroke="{ink or WHITE}" d="{poly(g["slash"])}"/></g>')


def tile(weight="regular", size=64):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 64 64">'
            f'<rect width="64" height="64" rx="14" fill="{BLACK}"/>{mark_svg_inner(weight)}</svg>')


# ---- Hand-fitted pixel masters (16/20/24/32) ---------------------------------
# chevron: tip x (left), arm length dx, rows y0..y1, thickness t (right is mirrored)
# studs: (x, w, top, bottom) in pixels; top row gets soft corners.
PIXEL = {
    16: dict(rx=3.5, tip=1, dx=3, y0=4, y1=11, t=2, studs=[(6, 1, 8, 10), (7, 1, 6, 9), (8, 1, 4, 8)]),
    20: dict(rx=4.5, tip=1, dx=4, y0=5, y1=14, t=2, studs=[(8, 1, 10, 13), (9, 1, 8, 12), (10, 1, 6, 11)]),
    24: dict(rx=5.5, tip=2, dx=4, y0=6, y1=17, t=2, studs=[(9, 2, 13, 17), (11, 2, 9, 15), (13, 2, 6, 12)]),
    32: dict(rx=7, tip=3, dx=5, y0=8, y1=23, t=3, studs=[(11, 3, 18, 22), (14, 3, 14, 20), (17, 3, 9, 16)]),
}


def chevron_pixels(n, tip, dx, y0, y1, t):
    px = set()
    mid = (y0 + y1) / 2
    half = (y1 - y0) / 2
    for y in range(y0, y1 + 1):
        x = tip + round(abs(y - mid) / half * dx)
        for i in range(t):
            px.add((x + i, y))
            px.add((n - 1 - (x + i), y))
    return px


def pixel_tile(n):
    c = PIXEL[n]
    px = chevron_pixels(n, c["tip"], c["dx"], c["y0"], c["y1"], c["t"])
    body = "".join(f'<rect x="{x}" y="{y}" width="1" height="1"/>' for x, y in sorted(px))
    st = ""
    for (x, w, top, bottom), ink in zip(c["studs"], STUD_INKS):
        st += f'<g fill="{ink}"><rect x="{x}" y="{top + 1}" width="{w}" height="{bottom - top - 1}"/>'
        if w >= 3:  # round top: corner pixels left empty, no off-palette blends
            st += f'<rect x="{x + 1}" y="{top}" width="{w - 2}" height="1"/>'
        else:
            st += f'<rect x="{x}" y="{top}" width="{w}" height="1"/>'
        st += "</g>"
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{n}" height="{n}" viewBox="0 0 {n} {n}" shape-rendering="crispEdges">'
            f'<rect width="{n}" height="{n}" rx="{c["rx"]}" fill="{BLACK}" shape-rendering="geometricPrecision"/>'
            f'<g fill="{SILVER}">{body}</g>{st}</svg>')

MARK_BOX = "4 12 56 40"


def mark_only(ink=None):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{MARK_BOX}">{mark_svg_inner(ink=ink)}</svg>'


# ---- Wordmark: Archivo wght 850 / wdth 118, shaped and outlined -------------
def wordmark_path(text="brickstrap", tracking=-0.03):
    face = hb.Face(hb.Blob.from_file_path(FONT))
    font = hb.Font(face)
    buf = hb.Buffer()
    buf.add_str(text)
    buf.guess_segment_properties()
    hb.shape(font, buf, {"kern": True, "liga": True})
    tt = TTFont(FONT)
    gs, order = tt.getGlyphSet(), tt.getGlyphOrder()
    pen = SVGPathPen(gs)
    x = 0
    for info, pos in zip(buf.glyph_infos, buf.glyph_positions):
        gs[order[info.codepoint]].draw(TransformPen(pen, (1, 0, 0, -1, x + pos.x_offset, 0)))
        x += pos.x_advance + tracking * face.upem
    x -= tracking * face.upem
    return pen.getCommands(), x, tt["glyf"]["b"].yMax, -tt["glyf"]["p"].yMin, tt["OS/2"].sxHeight


WM, WM_W, ASC, DESC, XH = wordmark_path()


def wordmark(fill):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 {-ASC} {WM_W:.0f} {ASC + DESC}">'
            f'<path fill="{fill}" d="{WM}"/></svg>')


def lockup(word_fill, with_tile=True, ink=None, ground=None):
    # The ascender of b/k reaches the mark's top (y=19: chevron caps and the white
    # stud), the baseline sits on its bottom (y=45).
    top, base = 19, 45
    s = (base - top) / ASC
    gap = 16
    if with_tile:
        lead = f'<rect width="64" height="64" rx="14" fill="{BLACK}"/>{mark_svg_inner()}'
        x_word, y0, h = 64 + gap, 0, 64
    else:
        lead = f'<g transform="translate(-4.5 0)">{mark_svg_inner(ink=ink)}</g>'
        x_word, y0, h = 51.5 + gap, 12, 40
    W = x_word + WM_W * s
    bg = ""
    if ground:
        pad = 24
        vb = f"{-pad} {y0 - pad} {W + 2 * pad:.1f} {h + 2 * pad}"
        bg = f'<rect x="{-pad}" y="{y0 - pad}" width="{W + 2 * pad:.1f}" height="{h + 2 * pad}" fill="{ground}"/>'
    else:
        vb = f"0 {y0} {W:.1f} {h}"
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">{bg}{lead}'
            f'<path fill="{word_fill}" transform="translate({x_word} {base}) scale({s:.5f})" d="{WM}"/></svg>')


svgs = {
    "brickstrap-icon.svg": tile(),
    "brickstrap-mark-dark.svg": mark_only(),
    "brickstrap-mark-black.svg": mark_only(BLACK),
    "brickstrap-mark-silver.svg": mark_only(SILVER),
    "brickstrap-mark-white.svg": mark_only(WHITE),
    "brickstrap-wordmark-black.svg": wordmark(BLACK),
    "brickstrap-wordmark-white.svg": wordmark(WHITE),
    "brickstrap-lockup-light.svg": lockup(BLACK),
    "brickstrap-lockup-light-mono.svg": lockup(BLACK, with_tile=False, ink=BLACK),
    "brickstrap-lockup-dark.svg": lockup(WHITE, with_tile=False),
    "brickstrap-lockup-dark-bg.svg": lockup(WHITE, with_tile=False, ground=BLACK),
}
for old in OUT.glob("*.svg"):
    old.unlink()
for name, s in svgs.items():
    (OUT / name).write_text(s)


# ---- Raster exports with provenance -----------------------------------------
def export(svg_name, png_name, w=None, h=None):
    out = PNG / png_name
    args = ["rsvg-convert", str(OUT / svg_name), "-o", str(out)]
    if w: args += ["-w", str(w)]
    if h: args += ["-h", str(h)]
    subprocess.run(args, check=True)
    meta = PngImagePlugin.PngInfo()
    meta.add_text("Source", f"logo/{svg_name}")
    meta.add_text("Software", "build_logo.py (hand-authored vector, rendered with rsvg-convert; no image generation)")
    meta.add_text("Comment", f"Brickstrap logo; contract {CONTRACT}")
    Image.open(out).save(out, pnginfo=meta)
    return out


for old in PNG.glob("*.png"):
    old.unlink()
ico_frames = []
for size in (16, 20, 24, 32, 40, 48, 64, 128, 256, 512, 1024, 2048):
    src = "brickstrap-icon.svg"
    p = export(src, f"brickstrap-icon-{size}.png", size, size)
    if size <= 256 and size != 128:
        ico_frames.append(str(p))
export("brickstrap-lockup-light.svg", "brickstrap-lockup-light.png", h=1024)
export("brickstrap-lockup-dark-bg.svg", "brickstrap-lockup-dark.png", h=1280)
subprocess.run(["convert", *ico_frames, str(OUT / "brickstrap.ico")], check=True)
print(f"wrote {len(svgs)} SVGs, {len(list(PNG.glob('*.png')))} PNGs, brickstrap.ico")
