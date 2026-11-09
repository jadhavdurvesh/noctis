#!/usr/bin/env python3
"""Generate the committed HTML named-character-reference table.

The data comes from Python's stdlib HTML5 entity table, which contains the
HTML named character references including legacy no-semicolon aliases.
This generator is a development tool; NOCTIS has no Python runtime dependency.
"""
from html.entities import html5
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates" / "noctis-html" / "src" / "named_entities.tsv"


def main() -> None:
    rows = []
    for name, value in sorted(html5.items()):
        codepoints = ",".join(f"{ord(character):x}" for character in value)
        rows.append(f"{name}\t{codepoints}")
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text("\n".join(rows) + "\n", encoding="utf-8")
    print(f"Generated {len(rows)} named character references: {OUTPUT}")


if __name__ == "__main__":
    main()
