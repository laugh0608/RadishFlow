"""Reproduce the frozen PR TP phase-label discrepancy without correcting outputs."""
import argparse
from pathlib import Path

import CoolProp.CoolProp as CP
from reference import new_state, query_saturation, validate_environment
from thermoml import write_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    validate_environment()
    rows = []
    for backend in ("PR", "HEOS"):
        for imposed in (False, True):
            state = new_state(backend, 0.5)
            if imposed:
                state.specify_phase(CP.iphase_gas)
            state.update(CP.PT_INPUTS, 200000.0, 300.0)
            rows.append({"backend": backend, "gas_phase_imposed_for_diagnosis_only": imposed,
                         "raw_phase": str(state.phase()), "raw_quality": state.Q(),
                         "density_kg_m3": state.rhomass(), "compressibility_factor": state.compressibility_factor()})
    write_json(args.output, {"temperature_k": 300.0, "pressure_pa": 200000.0, "z_propane": 0.5,
                            "queries": rows, "PR_bubble_pressure_pa": query_saturation("PR", 300.0, 0.0, 0.5)["pressure_pa"],
                            "PR_dew_pressure_pa": query_saturation("PR", 300.0, 1.0, 0.5)["pressure_pa"],
                            "policy": "Imposed gas branch is diagnostic only, never a replacement for frozen TP results or an acceptance oracle."})


if __name__ == "__main__":
    main()
