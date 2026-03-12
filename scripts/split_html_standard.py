#!/usr/bin/env python3

from __future__ import annotations

import re
import unicodedata
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "html-standard.txt"
OUTPUT_DIR = ROOT / "html-standard"
SECTION_RE = re.compile(
    r"^(?P<number>\d+(?:\.\d+)*) (?P<title>.+?)"
    r"(?P<url>https://html\.spec\.whatwg\.org/#\S+)\s*$"
)


@dataclass(frozen=True)
class Section:
    order: int
    number: str
    title: str
    url: str
    start_line: int
    end_line: int

    @property
    def filename(self) -> str:
        slug = slugify(self.title)
        return f"{self.order:04d}-{self.number}-{slug}.md"


def slugify(value: str, limit: int = 80) -> str:
    normalized = unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode("ascii")
    slug = re.sub(r"[^a-zA-Z0-9]+", "-", normalized.lower()).strip("-")
    slug = re.sub(r"-{2,}", "-", slug)
    if not slug:
        return "section"
    if len(slug) <= limit:
        return slug
    truncated = slug[:limit].rstrip("-")
    return truncated or "section"


def load_sections(lines: list[str]) -> list[Section]:
    headings: list[tuple[int, str, str, str]] = []
    for index, line in enumerate(lines):
        match = SECTION_RE.match(line)
        if not match:
            continue
        headings.append(
            (
                index,
                match.group("number"),
                match.group("title").strip(),
                match.group("url"),
            )
        )

    if not headings:
        raise RuntimeError("No numbered section headings were found in html-standard.txt.")

    sections: list[Section] = []
    for order, (start_index, number, title, url) in enumerate(headings, start=1):
        next_start = headings[order][0] if order < len(headings) else len(lines)
        sections.append(
            Section(
                order=order,
                number=number,
                title=title,
                url=url,
                start_line=start_index,
                end_line=next_start,
            )
        )
    return sections


def section_body(section: Section, lines: list[str]) -> str:
    body_lines = lines[section.start_line + 1 : section.end_line]

    while body_lines and not body_lines[0].strip():
        body_lines.pop(0)
    while body_lines and not body_lines[-1].strip():
        body_lines.pop()

    return "\n".join(body_lines).rstrip()


def write_section(section: Section, lines: list[str]) -> None:
    body = section_body(section, lines)
    target = OUTPUT_DIR / section.filename
    content = [
        f"# {section.number} {section.title}",
        "",
        f"[Source]({section.url})",
        "",
    ]
    if body:
        content.append(body)
        content.append("")

    target.write_text("\n".join(content), encoding="utf-8")


def write_index(sections: list[Section]) -> None:
    top_level = [section for section in sections if "." not in section.number]

    lines = [
        "# HTML Standard Split Markdown",
        "",
        f"Generated from [`html-standard.txt`](../html-standard.txt).",
        "",
        f"- Numbered sections: {len(sections)}",
        "- Output naming: `NNNN-section-number-section-title.md`",
        "- Scope: numbered sections only; non-numbered back matter such as Index/References is omitted.",
        "",
        "## Top-Level Chapters",
        "",
    ]

    for section in top_level:
        lines.append(f"- [{section.number} {section.title}](./{section.filename})")

    lines.append("")
    (OUTPUT_DIR / "README.md").write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    lines = SOURCE.read_text(encoding="utf-8").splitlines()
    sections = load_sections(lines)
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    for section in sections:
        write_section(section, lines)

    write_index(sections)
    print(f"Generated {len(sections)} markdown files in {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
