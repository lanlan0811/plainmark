"""从新图标源图生成 Tauri 所需的全套应用图标。

源图是 1254x1254 的白底图，内容（文档图形）只占中间 672x736，四周有大量留白。
处理三步：
  1. 从四角做连通域填充，把文档外的白底抠成透明——否则深色任务栏上会显示成
     一个白色方块。文档有灰色描边，填充会在描边处停住，不会吃掉内部。
  2. 裁出以内容为中心的正方形。
  3. 导出各尺寸的 PNG / ICO / ICNS。

用法：python scripts/build_icons.py
输出：src-tauri/icons/ 下的 PNG / ICO / ICNS
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "assets" / "icon" / "source.png"
OUT_DIR = ROOT / "src-tauri" / "icons"

# 内容相对源图的比例（由内容包围盒算出，见下方 detect_content_box）
PADDING_RATIO = 0.08  # 裁切后四周保留的留白比例


def detect_content_box(im: Image.Image, threshold: int = 12) -> tuple[int, int, int, int]:
    """找出与背景色不同的像素范围。

    源图是白底，图标内容为灰色/深色，按与背景的通道差判定。
    """
    w, h = im.size
    bg = im.getpixel((2, 2))
    px = im.load()

    minx, miny, maxx, maxy = w, h, 0, 0
    for y in range(h):
        for x in range(w):
            p = px[x, y]
            if max(abs(p[i] - bg[i]) for i in range(3)) > threshold:
                if x < minx:
                    minx = x
                if y < miny:
                    miny = y
                if x > maxx:
                    maxx = x
                if y > maxy:
                    maxy = y

    if maxx < minx or maxy < miny:
        raise SystemExit("未检测到内容，源图可能是纯色")
    return minx, miny, maxx, maxy


def remove_background(im: Image.Image, threshold: int = 20) -> Image.Image:
    """把文档外的背景抠成透明。

    从四角洪水填充标记外部区域。文档本身有灰色描边，填充会在此停住，
    因此文档内部的白色会被保留。
    """
    rgb = im.convert("RGB")
    w, h = rgb.size

    # 用洋红做标记色，避免与图中任何颜色冲突
    marker = (255, 0, 255)
    work = rgb.copy()

    for seed in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)]:
        if work.getpixel(seed) != marker:
            ImageDraw.floodfill(work, seed, marker, thresh=threshold)

    # 生成 alpha：被标记处透明
    alpha = Image.new("L", (w, h), 255)
    px_work = work.load()
    px_alpha = alpha.load()
    for y in range(h):
        for x in range(w):
            if px_work[x, y] == marker:
                px_alpha[x, y] = 0

    # 轻微羽化，消除硬边锯齿
    alpha = alpha.filter(ImageFilter.GaussianBlur(0.6))

    out = im.convert("RGBA")
    out.putalpha(alpha)
    return out


def square_crop(im: Image.Image, box: tuple[int, int, int, int]) -> Image.Image:
    """按内容包围盒裁出居中的正方形，四周留出等比例空白。"""
    x0, y0, x1, y1 = box
    cw, ch = x1 - x0 + 1, y1 - y0 + 1

    # 以较长边为基准，保证正方形能容纳全部内容
    side = max(cw, ch)
    side += round(side * PADDING_RATIO * 2)

    ccx, ccy = (x0 + x1) / 2, (y0 + y1) / 2
    left = round(ccx - side / 2)
    top = round(ccy - side / 2)

    w, h = im.size
    # 越界时退回到原图内可用的最大正方形
    if left < 0 or top < 0 or left + side > w or top + side > h:
        side = min(w, h, side)
        left = max(0, min(w - side, round(ccx - side / 2)))
        top = max(0, min(h - side, round(ccy - side / 2)))

    return im.crop((left, top, left + side, top + side))


def main() -> None:
    if not SOURCE.exists():
        raise SystemExit(f"缺少源图：{SOURCE}")

    raw = Image.open(SOURCE).convert("RGB")
    print(f"源图：{raw.size[0]}x{raw.size[1]}")

    # 1) 抠掉文档外的白底
    transparent = remove_background(raw)
    box = detect_content_box(raw)
    print(f"内容包围盒：{box} → {box[2]-box[0]+1}x{box[3]-box[1]+1}")

    # 2) 以内容为中心裁出正方形
    cropped = square_crop(transparent, box)
    print(f"裁切后：{cropped.size[0]}x{cropped.size[1]}")

    OUT_DIR.mkdir(parents=True, exist_ok=True)

    # 主图统一缩放到 1024，后续各尺寸都从它派生
    master = cropped.resize((1024, 1024), Image.Resampling.LANCZOS)

    master.save(OUT_DIR / "icon.png")
    master.resize((32, 32), Image.Resampling.LANCZOS).save(OUT_DIR / "32x32.png")
    master.resize((128, 128), Image.Resampling.LANCZOS).save(OUT_DIR / "128x128.png")
    master.resize((256, 256), Image.Resampling.LANCZOS).save(OUT_DIR / "128x128@2x.png")

    # Windows：多尺寸 ICO，小尺寸下系统会挑最接近的一档
    master.save(
        OUT_DIR / "icon.ico",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )

    # macOS：保留以便默认配置可用
    master.save(OUT_DIR / "icon.icns")

    print()
    for f in sorted(OUT_DIR.iterdir()):
        print(f"  {f.name:20} {f.stat().st_size / 1024:8.1f} KB")


if __name__ == "__main__":
    main()
