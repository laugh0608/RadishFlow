"""Read-only access to the accepted C2-A baseline and evidence serialization."""
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent
BASELINE = ROOT.parent / "c2a-pr76"
sys.path.insert(0, str(BASELINE))
from thermoml import load_json, sha256, write_json  # noqa: E402
import reference  # noqa: E402


def verify_baseline():
    protocol = load_json(ROOT / "protocol.json")
    if sha256(BASELINE / "manifest.json") != protocol["baseline_manifest_sha256"]:
        raise ValueError("Accepted baseline manifest changed")
    manifest = load_json(BASELINE / "manifest.json")
    for name, digest in manifest["repository_relative_sha256"].items():
        if sha256(ROOT.parents[2] / name) != digest:
            raise ValueError(f"Accepted baseline file changed: {name}")
