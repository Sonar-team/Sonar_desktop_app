#!/usr/bin/env python3
"""Summarize the packet and byte totals of an SFMS matrix CSV."""

from __future__ import annotations

import argparse
import csv
from pathlib import Path
from typing import Iterator, TextIO


REQUIRED_COLUMNS = {"count", "total_bytes"}


def data_lines(source: TextIO) -> Iterator[str]:
    for line in source:
        if not line.startswith("#"):
            yield line


def summarize(csv_path: Path) -> tuple[int, int, int]:
    with csv_path.open(encoding="utf-8-sig", newline="") as source:
        reader = csv.DictReader(data_lines(source))
        fieldnames = set(reader.fieldnames or ())
        missing = REQUIRED_COLUMNS - fieldnames
        if missing:
            columns = ", ".join(sorted(missing))
            raise ValueError(f"missing required CSV columns: {columns}")

        rows = 0
        packet_count = 0
        byte_count = 0
        for line_number, row in enumerate(reader, start=2):
            if not any(value for value in row.values() if value is not None):
                continue
            try:
                packet_count += int(row["count"] or "")
                byte_count += int(row["total_bytes"] or "")
            except (TypeError, ValueError) as error:
                raise ValueError(
                    f"invalid count or total_bytes at CSV data line {line_number}"
                ) from error
            rows += 1

    return rows, packet_count, byte_count


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("csv_path", type=Path)
    args = parser.parse_args()

    try:
        rows, packet_count, byte_count = summarize(args.csv_path)
    except (OSError, ValueError) as error:
        parser.error(str(error))

    print(rows, packet_count, byte_count)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
