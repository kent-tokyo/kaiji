#!/usr/bin/env python3
"""Generate kaiji variant map stubs from Unicode IVD sequences.

The IVD (Ideographic Variation Database) lists every base character that has
a registered Ideographic Variation Sequence. This script reads the IVD file,
groups base characters by collection, and emits candidate Rust pair entries
as a starting point for extending src/variants.rs.

NOTE: IVS defines *glyph* variants for the same base character. A different
use-case from kaiji's *semantic* variant map (e.g. 齋 → 斉). Review every
suggested entry before adding it; not all IVS base characters need a mapping.

Usage:
    # Download IVD_Sequences.txt once:
    curl -O https://www.unicode.org/Public/UCD/latest/ucd/IVD_Sequences.txt

    # Print candidates (default: Adobe-Japan1 collection):
    python3 scripts/gen_ivs_map.py --ivd IVD_Sequences.txt

    # Print all collections:
    python3 scripts/gen_ivs_map.py --ivd IVD_Sequences.txt --collection all

    # Emit Rust pair syntax for a specific collection:
    python3 scripts/gen_ivs_map.py --ivd IVD_Sequences.txt --rust
"""

import argparse
import sys
import urllib.request
from pathlib import Path


IVD_URL = "https://www.unicode.org/Public/UCD/latest/ucd/IVD_Sequences.txt"
DEFAULT_COLLECTION = "Adobe-Japan1"


def parse_ivd(lines: list[str]) -> dict[str, list[int]]:
    """Return {collection: [sorted base codepoints]} parsed from IVD lines."""
    result: dict[str, list[int]] = {}
    for line in lines:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split(";")
        if len(parts) < 3:
            continue
        seq_hex = parts[0].strip()
        collection = parts[1].strip()
        codepoints = [int(h, 16) for h in seq_hex.split()]
        if not codepoints:
            continue
        base = codepoints[0]  # first codepoint is the base character
        result.setdefault(collection, [])
        if base not in result[collection]:
            result[collection].append(base)
    for v in result.values():
        v.sort()
    return result


def fetch_ivd(path: Path | None) -> list[str]:
    if path and path.exists():
        return path.read_text(encoding="utf-8").splitlines()
    print(f"Downloading IVD from {IVD_URL} ...", file=sys.stderr)
    with urllib.request.urlopen(IVD_URL) as r:
        return r.read().decode("utf-8").splitlines()


def emit_rust(base_codepoints: list[int]) -> None:
    """Print Rust pair stubs. The canonical target is left as '?' for review."""
    print("// AUTO-GENERATED stubs — review and fill in canonical targets")
    print("// Format: ('variant', 'canonical'),")
    for cp in base_codepoints:
        ch = chr(cp)
        print(f"        ('{ch}', '?'), // U+{cp:04X}")


def emit_report(collection: str, base_codepoints: list[int]) -> None:
    print(f"Collection: {collection}  ({len(base_codepoints)} base characters)\n")
    cols = 8
    chars = [chr(cp) for cp in base_codepoints]
    for i in range(0, len(chars), cols):
        row = "  ".join(
            f"{c} U+{cp:04X}" for c, cp in zip(chars[i:i+cols], base_codepoints[i:i+cols])
        )
        print(" ", row)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--ivd", metavar="FILE", type=Path,
                        help="Path to IVD_Sequences.txt (downloaded if omitted)")
    parser.add_argument("--collection", default=DEFAULT_COLLECTION,
                        help=f"IVD collection name (default: {DEFAULT_COLLECTION}, or 'all')")
    parser.add_argument("--rust", action="store_true",
                        help="Emit Rust pair stubs instead of a readable report")
    args = parser.parse_args()

    lines = fetch_ivd(args.ivd)
    data = parse_ivd(lines)

    if not data:
        print("ERROR: No IVD entries parsed. Is the file valid?", file=sys.stderr)
        sys.exit(1)

    if args.collection == "all":
        collections = sorted(data)
    else:
        if args.collection not in data:
            available = ", ".join(sorted(data))
            print(f"ERROR: collection '{args.collection}' not found.", file=sys.stderr)
            print(f"Available: {available}", file=sys.stderr)
            sys.exit(1)
        collections = [args.collection]

    for col in collections:
        cps = data[col]
        if args.rust:
            print(f"\n// --- {col} ---")
            emit_rust(cps)
        else:
            emit_report(col, cps)
            print()


if __name__ == "__main__":
    main()
