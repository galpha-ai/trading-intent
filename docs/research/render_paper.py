from __future__ import annotations

import re
import subprocess
import textwrap
from dataclasses import dataclass
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY, TA_LEFT
from reportlab.lib.pagesizes import LETTER
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import inch
from reportlab.lib.utils import ImageReader
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.cidfonts import UnicodeCIDFont
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate,
    Frame,
    Image,
    PageBreak,
    PageTemplate,
    Paragraph,
    Preformatted,
)


ROOT = Path(__file__).resolve().parents[2]
RESEARCH_DIR = Path(__file__).resolve().parent
FIGURE_TEX = RESEARCH_DIR / "figures" / "pipeline-figure.tex"
FIGURE_BUILD_DIR = RESEARCH_DIR / "figures" / "build"
FIGURE_PDF = FIGURE_BUILD_DIR / "pipeline-figure.pdf"
FIGURE_PNG = FIGURE_BUILD_DIR / "pipeline-figure.png"
OUTPUT_DIR = ROOT / "output" / "pdf"


@dataclass(frozen=True)
class PaperSpec:
    source: Path
    output: Path
    title: str
    abstract_label: str
    footer_label: str
    figure_heading_prefix: str
    figure_caption: str
    author: str = "Bill Sun"
    use_cjk_font: bool = False


ENGLISH_PAPER = PaperSpec(
    source=RESEARCH_DIR / "tim-paper.md",
    output=OUTPUT_DIR / "tim-paper.pdf",
    title="Trade Intent Models: A Semantic Interface for Cross-Market Trading and Agentic Strategy Execution",
    abstract_label="Abstract",
    footer_label="TIM Research Note",
    figure_heading_prefix="5. A System View:",
    figure_caption="Figure 1. TIM acts as the stable representation layer from idea ingestion to execution-policy selection and venue-native deployment.",
)

CHINESE_PAPER = PaperSpec(
    source=RESEARCH_DIR / "tim-paper.zh.md",
    output=OUTPUT_DIR / "tim-paper-zh.pdf",
    title="Trade Intent Models：跨市场交易与 Agent 策略执行的语义接口",
    abstract_label="摘要",
    footer_label="TIM 研究笔记",
    figure_heading_prefix="5. 系统视角：",
    figure_caption="图 1. TIM 作为稳定表示层，连接 idea ingestion、execution-policy selection 与 venue-native deployment。",
    use_cjk_font=True,
)

PAPERS = [ENGLISH_PAPER, CHINESE_PAPER]
PAPER_AUTHOR = "Bill Sun and Alpha.Dev team (https://alpha.dev/)"


def register_fonts(use_cjk_font: bool) -> dict[str, str]:
    if not use_cjk_font:
        return {
            "regular": "Helvetica",
            "bold": "Helvetica-Bold",
            "italic": "Helvetica-Oblique",
        }

    candidates = [
        ("SongtiSC", Path("/System/Library/Fonts/Supplemental/Songti.ttc")),
        ("HiraginoSansGB", Path("/System/Library/Fonts/Hiragino Sans GB.ttc")),
    ]
    for font_name, font_path in candidates:
        if not font_path.exists():
            continue
        try:
            pdfmetrics.registerFont(TTFont(font_name, str(font_path)))
            return {
                "regular": font_name,
                "bold": font_name,
                "italic": font_name,
            }
        except Exception:
            continue

    fallback_name = "STSong-Light"
    pdfmetrics.registerFont(UnicodeCIDFont(fallback_name))
    return {
        "regular": fallback_name,
        "bold": fallback_name,
        "italic": fallback_name,
    }


def build_styles(paper: PaperSpec):
    styles = getSampleStyleSheet()
    fonts = register_fonts(paper.use_cjk_font)
    word_wrap = "CJK" if paper.use_cjk_font else None
    return {
        "title": ParagraphStyle(
            "PaperTitle",
            parent=styles["Title"],
            fontName=fonts["bold"],
            fontSize=16.2,
            leading=18.8,
            alignment=TA_CENTER,
            textColor=colors.HexColor("#10233F"),
            spaceAfter=7,
            wordWrap=word_wrap,
        ),
        "author": ParagraphStyle(
            "PaperAuthor",
            parent=styles["Normal"],
            fontName=fonts["regular"],
            fontSize=9.5,
            leading=11.2,
            alignment=TA_CENTER,
            textColor=colors.HexColor("#425466"),
            spaceAfter=11,
            wordWrap=word_wrap,
        ),
        "h2": ParagraphStyle(
            "PaperHeading2",
            parent=styles["Heading2"],
            fontName=fonts["bold"],
            fontSize=11.1,
            leading=12.7,
            textColor=colors.HexColor("#10233F"),
            spaceBefore=5,
            spaceAfter=3,
            wordWrap=word_wrap,
        ),
        "body": ParagraphStyle(
            "PaperBody",
            parent=styles["BodyText"],
            fontName=fonts["regular"],
            fontSize=8.9,
            leading=11.5,
            alignment=TA_JUSTIFY,
            textColor=colors.HexColor("#1F2933"),
            spaceAfter=4,
            wordWrap=word_wrap,
        ),
        "abstract_label": ParagraphStyle(
            "AbstractLabel",
            parent=styles["BodyText"],
            fontName=fonts["bold"],
            fontSize=8.9,
            leading=11.5,
            alignment=TA_LEFT,
            textColor=colors.HexColor("#10233F"),
            spaceAfter=3,
            wordWrap=word_wrap,
        ),
        "caption": ParagraphStyle(
            "FigureCaption",
            parent=styles["BodyText"],
            fontName=fonts["italic"],
            fontSize=7.8,
            leading=9.1,
            alignment=TA_LEFT,
            textColor=colors.HexColor("#52606D"),
            spaceBefore=4,
            spaceAfter=5,
            wordWrap=word_wrap,
        ),
        "code": ParagraphStyle(
            "CodeStyle",
            parent=styles["Code"],
            fontName="Courier",
            fontSize=7.0,
            leading=8.3,
            leftIndent=4,
            rightIndent=4,
            textColor=colors.HexColor("#243B53"),
            backColor=colors.HexColor("#F4F7FB"),
            borderWidth=0.5,
            borderColor=colors.HexColor("#D9E2EC"),
            borderPadding=6,
            spaceBefore=4,
            spaceAfter=8,
        ),
    }


def draw_page(paper: PaperSpec):
    fonts = register_fonts(paper.use_cjk_font)

    def _draw_page(canvas, doc):
        canvas.saveState()
        canvas.setTitle(paper.title)
        canvas.setAuthor(PAPER_AUTHOR)
        width, height = LETTER
        left = doc.leftMargin
        right = width - doc.rightMargin
        top = height - 0.62 * inch
        bottom = 0.55 * inch

        canvas.setStrokeColor(colors.HexColor("#D9E2EC"))
        canvas.setLineWidth(0.6)
        canvas.line(left, top, right, top)
        canvas.line(left, bottom, right, bottom)

        canvas.setFillColor(colors.HexColor("#52606D"))
        canvas.setFont(fonts["regular"], 8)
        canvas.drawString(left, bottom - 0.18 * inch, paper.footer_label)
        page_label = f"Page {canvas.getPageNumber()}"
        canvas.drawRightString(right, bottom - 0.18 * inch, page_label)
        canvas.restoreState()

    return _draw_page


def paragraph_text(raw: str) -> str:
    text = raw.strip()
    text = text.replace("&", "&amp;")
    text = text.replace("<", "&lt;").replace(">", "&gt;")
    text = re.sub(r"`([^`]+)`", r"<font name='Courier'>\1</font>", text)
    return text


def parse_sections(markdown: str):
    blocks: list[tuple[str, str]] = []
    lines = markdown.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i].rstrip()
        if not line:
            i += 1
            continue

        if line.startswith("# "):
            blocks.append(("title", line[2:].strip()))
            i += 1
            continue

        if line.startswith("## "):
            blocks.append(("heading", line[3:].strip()))
            i += 1
            continue

        if line == "---":
            blocks.append(("pagebreak", ""))
            i += 1
            continue

        if line.startswith("1. ") or re.match(r"^\d+\.\s", line):
            numbered = [line]
            i += 1
            while i < len(lines) and re.match(r"^\d+\.\s", lines[i].strip()):
                numbered.append(lines[i].strip())
                i += 1
            blocks.append(("numbered", "\n".join(numbered)))
            continue

        if line.startswith("`") and line.endswith("`") and len(line) > 1:
            blocks.append(("code", line.strip("`")))
            i += 1
            continue

        para = [line]
        i += 1
        while i < len(lines):
            nxt = lines[i].rstrip()
            if not nxt:
                break
            if nxt.startswith("# ") or nxt.startswith("## ") or nxt == "---":
                break
            if re.match(r"^\d+\.\s", nxt.strip()):
                break
            para.append(nxt)
            i += 1
        blocks.append(("paragraph", " ".join(p.strip() for p in para)))
    return blocks


def ensure_pipeline_figure() -> Path:
    FIGURE_BUILD_DIR.mkdir(parents=True, exist_ok=True)

    needs_render = (
        not FIGURE_PNG.exists()
        or FIGURE_PNG.stat().st_mtime < FIGURE_TEX.stat().st_mtime
    )
    if not needs_render:
        return FIGURE_PNG

    subprocess.run(
        [
            "tectonic",
            "--outdir",
            str(FIGURE_BUILD_DIR),
            str(FIGURE_TEX),
        ],
        check=True,
        cwd=FIGURE_TEX.parent,
    )
    subprocess.run(
        [
            "pdftoppm",
            "-singlefile",
            "-png",
            str(FIGURE_PDF),
            str(FIGURE_PNG.with_suffix("")),
        ],
        check=True,
    )
    return FIGURE_PNG


def compile_pipeline_figure():
    figure_path = ensure_pipeline_figure()
    image_reader = ImageReader(str(figure_path))
    width_px, height_px = image_reader.getSize()
    target_width = 5.7 * inch
    target_height = target_width * height_px / width_px
    return Image(str(figure_path), width=target_width, height=target_height)


def build_story(paper: PaperSpec):
    styles = build_styles(paper)
    content = paper.source.read_text(encoding="utf-8")
    blocks = parse_sections(content)
    story = []

    title_seen = False
    author_seen = False

    for block_type, value in blocks:
        if block_type == "title":
            story.append(Paragraph(paragraph_text(value), styles["title"]))
            title_seen = True
            continue

        if title_seen and not author_seen and block_type == "paragraph":
            story.append(Paragraph(paragraph_text(value), styles["author"]))
            author_seen = True
            continue

        if block_type == "heading":
            if value.lower() == "abstract" or value == "摘要":
                story.append(Paragraph(paper.abstract_label, styles["abstract_label"]))
            else:
                story.append(Paragraph(paragraph_text(value), styles["h2"]))
                if value.startswith(paper.figure_heading_prefix):
                    story.append(compile_pipeline_figure())
                    story.append(
                        Paragraph(
                            paper.figure_caption,
                            styles["caption"],
                        )
                    )
            continue

        if block_type == "paragraph":
            story.append(Paragraph(paragraph_text(value), styles["body"]))
            continue

        if block_type == "numbered":
            items = []
            for line in value.splitlines():
                num, text = line.split(". ", 1)
                items.append(f"<b>{num}.</b> {paragraph_text(text)}")
            story.append(Paragraph("<br/>".join(items), styles["body"]))
            continue

        if block_type == "code":
            wrapped = textwrap.fill(value, width=88, break_long_words=False, break_on_hyphens=False)
            story.append(Preformatted(wrapped, styles["code"]))
            continue

        if block_type == "pagebreak":
            story.append(PageBreak())

    return story

def render_paper(paper: PaperSpec):
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    author = PAPER_AUTHOR

    doc = BaseDocTemplate(
        str(paper.output),
        pagesize=LETTER,
        leftMargin=0.72 * inch,
        rightMargin=0.72 * inch,
        topMargin=0.86 * inch,
        bottomMargin=0.75 * inch,
        title=paper.title,
        author=author,
    )

    frame = Frame(
        doc.leftMargin,
        doc.bottomMargin,
        doc.width,
        doc.height,
        leftPadding=0,
        bottomPadding=0,
        rightPadding=0,
        topPadding=0,
        id="paper-frame",
    )
    template = PageTemplate(id=f"paper-{paper.output.stem}", frames=[frame], onPage=draw_page(paper))
    doc.addPageTemplates([template])
    doc.build(build_story(paper))


def main():
    for paper in PAPERS:
        render_paper(paper)


if __name__ == "__main__":
    main()
