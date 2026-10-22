"""Extract the selected NIST ThermoML measurements without fitting or interpolation."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SPECS = {
    "je800377b": ("10.1021/je800377b", [1, 2, 3, 4]),
    "je049671t": ("10.1021/je049671t", [1, 2, 3]),
    "j.jct.2007.06.014": ("10.1016/j.jct.2007.06.014", [2]),
    "je900124n": ("10.1021/je900124n", [1, 2]),
    "j.jct.2006.07.012": ("10.1016/j.jct.2006.07.012", [1]),
    "j.jct.2006.09.014": ("10.1016/j.jct.2006.09.014", [1, 2, 3]),
}


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def content_item(mapping: dict):
    items = [(k, v) for k, v in mapping.items() if k != "tml_elements"]
    if len(items) != 1:
        raise ValueError(f"Expected a single ThermoML descriptor: {mapping}")
    return items[0]


def variable_descriptor(variable: dict) -> tuple[str, float]:
    identity = variable["VariableID"]
    kind, label = content_item(identity["VariableType"])
    if (kind, label) == ("eTemperature", "Temperature, K"):
        return "temperature_k", 1.0
    if (kind, label) == ("ePressure", "Pressure, kPa"):
        return "pressure_pa", 1000.0
    if (kind, label) == ("eComponentComposition", "Mole fraction"):
        if identity["RegNum"]["nOrgNum"] != 1 or variable["VarPhaseID"]["eVarPhase"] != "Liquid":
            raise ValueError("Selected binary composition must be propane in the liquid phase")
        return "x_propane", 1.0
    raise ValueError(f"Unsupported selected variable: {kind}, {label}")


def extract_dataset(source_id: str, dataset: dict, compounds: dict) -> dict:
    if dataset.get("Constraint") or len(dataset["Property"]) != 1:
        raise ValueError("Selected datasets must have one property and no hidden constraints")
    prop = dataset["Property"][0]
    group = content_item(prop["Property-MethodID"]["PropertyGroup"])[1]
    label = group["ePropName"]
    fields = {
        "Vapor or sublimation pressure, kPa": ("pressure_pa", 1000.0),
        "Mass density, kg/m3": ("density_kg_m3", 1.0),
        "Mole fraction": ("y_propane", 1.0),
    }
    field, factor = fields[label]
    if field == "y_propane":
        if prop["Property-MethodID"]["RegNum"]["nOrgNum"] != 1 or prop["PropPhaseID"]["ePropPhase"] != "Gas":
            raise ValueError("VLE property must be propane vapor mole fraction")
    variables = {v["nVarNumber"]: variable_descriptor(v) for v in dataset["Variable"]}
    number = dataset["nPureOrMixtureDataNumber"]
    rows = []
    for index, point in enumerate(dataset["NumValues"], 1):
        values = point["PropertyValue"]
        if len(values) != 1 or values[0]["nPropNumber"] != prop["nPropNumber"]:
            raise ValueError("Unexpected property identity")
        observed_variables = point["VariableValue"]
        if {v["nVarNumber"] for v in observed_variables} != set(variables):
            raise ValueError("Missing or extra ThermoML variable")
        row = {"id": f"{source_id}/d{number}/p{index}", "point_number": index}
        for value in observed_variables:
            name, scale = variables[value["nVarNumber"]]
            row[name] = value["nVarValue"] * scale
        row[field] = values[0]["nPropValue"] * factor
        uncertainty = values[0].get("CombinedUncertainty", {})
        value = uncertainty.get("nCombExpandUncertValue")
        row["expanded_uncertainty"] = None if value is None else value * factor
        row["reported_significant_digits"] = values[0]["nPropDigits"]
        rows.append(row)
    return {
        "id": f"{source_id}/d{number}",
        "source_id": source_id,
        "dataset_number": number,
        "components": [compounds[c["RegNum"]["nOrgNum"]] for c in dataset["Component"]],
        "property": field,
        "original_property_label": label,
        "conversion_factor": factor,
        "property_phase": prop["PropPhaseID"]["ePropPhase"],
        "phases": [p["ePhase"] for p in dataset["PhaseID"]],
        "method": group.get("eMethodName", group.get("sMethodName")),
        "uncertainty": {k: v for k, v in prop.get("CombinedUncertainty", {}).items() if k != "tml_elements"},
        "rows": rows,
    }


def extract(cache: Path, expected_sources: dict | None) -> tuple[dict, dict]:
    sources, datasets = {}, []
    for name, (doi, selected) in SPECS.items():
        path = cache / f"{name}.json"
        digest = sha256(path)
        if expected_sources and digest != expected_sources[name]["sha256"]:
            raise ValueError(f"Source hash changed: {name}")
        document = load_json(path)
        citation = document["Citation"]
        if citation["sDOI"] != doi:
            raise ValueError(f"DOI mismatch: {name}")
        sources[name] = {
            "doi": doi,
            "url": f"https://trc.nist.gov/ThermoML/{doi}.json",
            "sha256": digest,
            "thermoml_xml_md5": document["THERMOML_MD5_CHECKSUM"],
            "archive_extracted_on": citation["dateCit"],
            "retrieved_on": "2026-10-22",
            "title": citation["sTitle"],
            "authors": citation["sAuthor"],
            "selected_datasets": selected,
            "license": "NIST ThermoML/Data Archive mds2-2422 metadata: https://www.nist.gov/open/license",
            "transform": "Selected numerical datasets only; kPa to Pa; retain point IDs and compiler-expanded uncertainties; no abstract or article text copied.",
        }
        compounds = {}
        for compound in document["Compound"]:
            number = compound["RegNum"]["nOrgNum"]
            names = compound["sCommonName"]
            if "propane" in names:
                compounds[number] = "Propane"
            elif "butane" in names:
                compounds[number] = "n-Butane"
        selected_datasets = [d for d in document["PureOrMixtureData"] if d["nPureOrMixtureDataNumber"] in selected]
        if len(selected_datasets) != len(selected):
            raise ValueError(f"Dataset missing: {name}")
        datasets.extend(extract_dataset(name, d, compounds) for d in selected_datasets)
    return sources, {"schema": "radishflow.c2a-experiments", "version": 1, "datasets": datasets}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--fetch", action="store_true", help="Download the six public NIST source files into the explicit cache")
    parser.add_argument("--freeze", action="store_true", help="Initial source freeze only; refuses to overwrite")
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    args.source_dir.mkdir(parents=True, exist_ok=True)
    if args.fetch:
        for name, (doi, _) in SPECS.items():
            subprocess.run(["curl", "--fail", "--location", "--silent", "--show-error", "--max-time", "60",
                            f"https://trc.nist.gov/ThermoML/{doi}.json", "--output", str(args.source_dir / f"{name}.json")], check=True)
    sources_path = ROOT / "sources.json"
    if args.freeze and sources_path.exists():
        parser.error("sources.json is already frozen; do not silently replace the evidence")
    expected = None if args.freeze else load_json(sources_path)
    sources, data = extract(args.source_dir, expected)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    if args.freeze:
        write_json(sources_path, sources)
    write_json(args.output_dir / "experimental-data.json", data)
    print(f"Extracted {len(data['datasets'])} datasets / {sum(len(d['rows']) for d in data['datasets'])} measurements")


if __name__ == "__main__":
    main()
