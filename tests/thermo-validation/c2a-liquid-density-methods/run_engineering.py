"""Transport numeric fixtures through the unchanged PR/HEOS-labelled S1/S2 harness.

The legacy backend column is printed metadata, never a calculation selector.
Use temporary transport IDs and restore the recorded candidate identity only
after exact output coverage checks. No PR thermodynamics run in this harness.
"""

from __future__ import annotations

import argparse
import csv
import io
import subprocess
from pathlib import Path


def run(inputs: Path, executable: Path, output: Path) -> None:
    with inputs.open(encoding="utf-8", newline="") as file:
        reader = csv.DictReader(file, delimiter="\t")
        fields, rows = reader.fieldnames, list(reader)
    identities = [(r["id"], r["backend"]) for r in rows]
    if len(set(identities)) != len(identities):
        raise ValueError("Duplicate engineering input")
    wire = io.StringIO()
    writer = csv.DictWriter(wire, fields, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    for i, row in enumerate(rows):
        writer.writerow(dict(row, id=f"transport-{i}", backend="PR"))
    result = subprocess.run([str(executable.resolve())], input=wire.getvalue(), text=True, capture_output=True, check=True)
    reader = csv.DictReader(io.StringIO(result.stdout), delimiter="\t")
    output_fields, values = reader.fieldnames, list(reader)
    expected = [(f"transport-{i}", "PR") for i in range(len(rows))]
    if [(r["id"], r["backend"]) for r in values] != expected:
        raise ValueError("Harness returned unexpected coverage or order")
    with output.open("w", encoding="utf-8", newline="") as file:
        writer = csv.DictWriter(file, output_fields, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        for identity, row in zip(identities, values):
            writer.writerow(dict(row, id=identity[0], backend=identity[1]))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inputs", required=True, type=Path)
    parser.add_argument("--harness", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    run(args.inputs, args.harness, args.output)
