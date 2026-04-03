from __future__ import annotations

import re
import textwrap
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY, TA_LEFT
from reportlab.lib.pagesizes import LETTER
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import inch
from reportlab.platypus import (
    BaseDocTemplate,
    Frame,
    PageBreak,
    PageTemplate,
    Paragraph,
    Preformatted,
    Spacer,
    Table,
    TableStyle,
)


ROOT = Path(__file__).resolve().parents[2]
SOURCE = Path(__file__).with_name("tim-paper.md")
OUTPUT_DIR = ROOT / "output" / "pdf"
OUTPUT_PDF = OUTPUT_DIR / "tim-paper.pdf"
PAPER_TITLE = "Trade Intent Models: A Semantic Interface for Cross-Market Trading and Agentic Strategy Execution"


def build_styles():
    styles = getSampleStyleSheet()
    return {
        "title": ParagraphStyle(
            "PaperTitle",
            parent=styles["Title"],
            fontName="Helvetica-Bold",
            fontSize=17,
            leading=20,
            alignment=TA_CENTER,
            textColor=colors.HexColor("#10233F"),
            spaceAfter=8,
        ),
        "author": ParagraphStyle(
            "PaperAuthor",
            parent=styles["Normal"],
            fontName="Helvetica",
            fontSize=10,
            leading=12,
            alignment=TA_CENTER,
            textColor=colors.HexColor("#425466"),
            spaceAfter=14,
        ),
        "h2": ParagraphStyle(
            "PaperHeading2",
            parent=styles["Heading2"],
            fontName="Helvetica-Bold",
            fontSize=11.5,
            leading=13.2,
            textColor=colors.HexColor("#10233F"),
            spaceBefore=6,
            spaceAfter=4,
        ),
        "body": ParagraphStyle(
            "PaperBody",
            parent=styles["BodyText"],
            fontName="Helvetica",
            fontSize=9.1,
            leading=12.2,
            alignment=TA_JUSTIFY,
            textColor=colors.HexColor("#1F2933"),
            spaceAfter=5,
        ),
        "abstract_label": ParagraphStyle(
            "AbstractLabel",
            parent=styles["BodyText"],
            fontName="Helvetica-Bold",
            fontSize=9.1,
            leading=12.2,
            alignment=TA_LEFT,
            textColor=colors.HexColor("#10233F"),
            spaceAfter=4,
        ),
        "caption": ParagraphStyle(
            "FigureCaption",
            parent=styles["BodyText"],
            fontName="Helvetica-Oblique",
            fontSize=8.1,
            leading=9.7,
            alignment=TA_LEFT,
            textColor=colors.HexColor("#52606D"),
            spaceBefore=5,
            spaceAfter=6,
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


def draw_page(canvas, doc):
    canvas.saveState()
    canvas.setTitle(PAPER_TITLE)
    canvas.setAuthor("Bill Sun")
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
    canvas.setFont("Helvetica", 8)
    canvas.drawString(left, bottom - 0.18 * inch, "TIM Research Note")
    page_label = f"Page {canvas.getPageNumber()}"
    canvas.drawRightString(right, bottom - 0.18 * inch, page_label)
    canvas.restoreState()


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


def compiler_pipeline_figure():
    rows = [
        [
            "Natural Language",
            "->",
            "High-Level Trade Intent",
            "->",
            "Execution Task Graph",
            "->",
            "Venue Instructions",
        ],
        [
            "Ambiguous human request",
            "",
            "Unambiguous, typed, cross-venue IR",
            "",
            "TWAP, VWAP, AMM splits, routing",
            "",
            "Adapter output",
        ],
    ]
    table = Table(
        rows,
        colWidths=[1.18 * inch, 0.2 * inch, 1.9 * inch, 0.2 * inch, 1.9 * inch, 0.2 * inch, 1.42 * inch],
        hAlign="LEFT",
    )
    table.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (0, 1), colors.HexColor("#EAF2FF")),
                ("BACKGROUND", (2, 0), (2, 1), colors.HexColor("#E8FFF5")),
                ("BACKGROUND", (4, 0), (4, 1), colors.HexColor("#FFF7E8")),
                ("BACKGROUND", (6, 0), (6, 1), colors.HexColor("#F3E8FF")),
                ("SPAN", (0, 0), (0, 0)),
                ("SPAN", (2, 0), (2, 0)),
                ("SPAN", (4, 0), (4, 0)),
                ("SPAN", (6, 0), (6, 0)),
                ("TEXTCOLOR", (0, 0), (-1, -1), colors.HexColor("#10233F")),
                ("FONTNAME", (0, 0), (-1, 0), "Helvetica-Bold"),
                ("FONTNAME", (0, 1), (-1, 1), "Helvetica"),
                ("FONTSIZE", (0, 0), (-1, -1), 8.2),
                ("LEADING", (0, 0), (-1, -1), 10),
                ("BOX", (0, 0), (0, 1), 0.6, colors.HexColor("#9FB3C8")),
                ("BOX", (2, 0), (2, 1), 0.6, colors.HexColor("#7BC6A4")),
                ("BOX", (4, 0), (4, 1), 0.6, colors.HexColor("#D9B06B")),
                ("BOX", (6, 0), (6, 1), 0.6, colors.HexColor("#B89AE6")),
                ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                ("ALIGN", (1, 0), (1, -1), "CENTER"),
                ("ALIGN", (3, 0), (3, -1), "CENTER"),
                ("ALIGN", (5, 0), (5, -1), "CENTER"),
                ("FONTNAME", (1, 0), (5, 1), "Helvetica-Bold"),
                ("TEXTCOLOR", (1, 0), (5, 1), colors.HexColor("#52606D")),
                ("TOPPADDING", (0, 0), (-1, -1), 5),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
                ("LEFTPADDING", (0, 0), (-1, -1), 6),
                ("RIGHTPADDING", (0, 0), (-1, -1), 6),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
            ]
        )
    )
    return table


def build_story():
    styles = build_styles()
    content = SOURCE.read_text()
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
            if value.lower() == "abstract":
                story.append(Paragraph("Abstract", styles["abstract_label"]))
            else:
                story.append(Paragraph(paragraph_text(value), styles["h2"]))
                if value.startswith("5. A System View: From Idea to Agentic Execution"):
                    story.append(compiler_pipeline_figure())
                    story.append(
                        Paragraph(
                            "Figure 1. TIM acts as the stable representation layer from idea ingestion to execution-policy selection and venue-native deployment.",
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

def main():
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    doc = BaseDocTemplate(
        str(OUTPUT_PDF),
        pagesize=LETTER,
        leftMargin=0.8 * inch,
        rightMargin=0.8 * inch,
        topMargin=0.9 * inch,
        bottomMargin=0.78 * inch,
        title=PAPER_TITLE,
        author="Bill Sun",
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
    template = PageTemplate(id="paper", frames=[frame], onPage=draw_page)
    doc.addPageTemplates([template])
    doc.build(build_story())


if __name__ == "__main__":
    main()
