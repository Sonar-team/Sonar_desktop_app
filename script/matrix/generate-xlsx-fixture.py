#!/usr/bin/env python3
"""Génère une fixture XLSX de matrice de flux à partir d'un CSV SFMS.

Aucune dépendance : un XLSX est un ZIP de XML, écrit ici à la main. Le
fichier produit est déterministe (dates figées, ordre stable), pour qu'une
régénération ne fasse pas bouger le dépôt sans raison.

Usage :
    script/matrix/generate-xlsx-fixture.py <entrée.csv> <sortie.xlsx>

Le préambule `#SFMS` du CSV, s'il est présent, devient la première ligne de
la feuille — exactement comme l'exporterait un tableur ayant ouvert le CSV.
"""

import csv
import sys
import zipfile
from pathlib import Path
from xml.sax.saxutils import escape

CONTENT_TYPES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"""

ROOT_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"""

WORKBOOK = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Matrice" sheetId="1" r:id="rId1"/></sheets>
</workbook>"""

WORKBOOK_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"""


def column_name(index: int) -> str:
    """Index 0 -> 'A', 26 -> 'AA' (référence de cellule Excel)."""
    name = ""
    index += 1
    while index:
        index, rest = divmod(index - 1, 26)
        name = chr(ord("A") + rest) + name
    return name


def sheet_xml(rows: list[list[str]]) -> str:
    parts = [
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
        '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">',
        "<sheetData>",
    ]
    for row_index, row in enumerate(rows, start=1):
        parts.append(f'<row r="{row_index}">')
        for col_index, value in enumerate(row):
            ref = f"{column_name(col_index)}{row_index}"
            # `t="inlineStr"` : chaînes portées par la cellule, sans table
            # partagée — le lecteur n'a qu'une seule forme à décoder.
            parts.append(
                f'<c r="{ref}" t="inlineStr"><is><t xml:space="preserve">'
                f"{escape(value)}</t></is></c>"
            )
        parts.append("</row>")
    parts.append("</sheetData></worksheet>")
    return "".join(parts)


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2

    source, destination = Path(sys.argv[1]), Path(sys.argv[2])
    rows: list[list[str]] = []
    with source.open(newline="", encoding="utf-8") as handle:
        first = handle.readline()
        if first.startswith("#SFMS"):
            rows.append([first.rstrip("\n")])
        else:
            handle.seek(0)
        rows.extend(list(csv.reader(handle)))

    destination.parent.mkdir(parents=True, exist_ok=True)
    # Horodatage figé : deux générations produisent des octets identiques.
    fixed_date = (2026, 1, 1, 0, 0, 0)
    with zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, content in [
            ("[Content_Types].xml", CONTENT_TYPES),
            ("_rels/.rels", ROOT_RELS),
            ("xl/workbook.xml", WORKBOOK),
            ("xl/_rels/workbook.xml.rels", WORKBOOK_RELS),
            ("xl/worksheets/sheet1.xml", sheet_xml(rows)),
        ]:
            info = zipfile.ZipInfo(name, date_time=fixed_date)
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, content)

    print(f"{destination} : {len(rows)} ligne(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
