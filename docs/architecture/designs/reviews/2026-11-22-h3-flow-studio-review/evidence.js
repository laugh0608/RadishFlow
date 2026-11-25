window.h3Evidence = {
  "project": {
    "document_id": "h3-linked-review-pair",
    "project_schema_version": 4,
    "flow_revision": 1,
    "file_sha256": "2d841c1f4857d193b963571ac71e64421684e6ffbd9cf57e9526cc61e1995485",
    "read_only": false,
    "equipment_task_count": 2
  },
  "cases": [
    {
      "tag": "E-301",
      "id": "h3-water",
      "streams": {
        "cold": {
          "creation_id": "00000000-0000-4000-8000-000000000193",
          "stream_id": "cold-feed"
        },
        "hot": {
          "creation_id": "00000000-0000-4000-8000-000000000191",
          "stream_id": "hot-feed"
        }
      },
      "revisions": {
        "case": 12,
        "geometry": 7,
        "task": 6
      },
      "limits": {
        "max_cold_pressure_drop_pa": 25000.0,
        "max_hot_pressure_drop_pa": 11000.0,
        "min_heat_duty_w": 5500.0
      },
      "result": {
        "duty": 5521.707636437074,
        "ua": 874.5534231148622,
        "checks": {
          "cold_pressure_drop": "passed",
          "heat_duty": "passed",
          "hot_pressure_drop": "passed"
        },
        "hotDrop": 9799.896961973793,
        "coldDrop": 23679.13004047922,
        "hotOutlet": 297.91426274251074,
        "coldOutlet": 294.9825080401212,
        "hotInlet": {
          "molar_flow_mol_s": 14.0,
          "pressure_pa": 100000.0,
          "temperature_k": 303.15
        },
        "coldInlet": {
          "molar_flow_mol_s": 40.0,
          "pressure_pa": 100000.0,
          "temperature_k": 293.15
        },
        "flow": {
          "flow_revision": 1,
          "package": {
            "id": "h3-pure-water-lite-v1",
            "payload_fingerprint": "sha256:1d175d0a82c4d6f586219a32c1bdbad46555356e5d2d50eab3e6bd7a0cec0c26",
            "version": "1"
          },
          "run_id": "review",
          "scope": "request",
          "sequence": 1,
          "snapshot_id": "h3-linked-review-pair-rev-1-seq-1"
        },
        "qualification": {
          "id": "h3-pure-water",
          "phase_reference": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C7732185&Type=ANTOINE",
          "revision": 1
        }
      }
    },
    {
      "tag": "E-302",
      "id": "h3-water-50",
      "streams": {
        "cold": {
          "creation_id": "00000000-0000-4000-8000-0000000001f7",
          "stream_id": "cold-feed-50"
        },
        "hot": {
          "creation_id": "00000000-0000-4000-8000-0000000001f5",
          "stream_id": "hot-feed-50"
        }
      },
      "revisions": {
        "case": 12,
        "geometry": 7,
        "task": 6
      },
      "limits": {
        "max_cold_pressure_drop_pa": 25000.0,
        "max_hot_pressure_drop_pa": 11000.0,
        "min_heat_duty_w": 5500.0
      },
      "result": {
        "duty": 5877.489522370671,
        "ua": 945.0937788010181,
        "checks": {
          "cold_pressure_drop": "failed",
          "heat_duty": "passed",
          "hot_pressure_drop": "passed"
        },
        "hotDrop": 9799.896961973793,
        "coldDrop": 34962.54768526354,
        "hotOutlet": 297.57690687393307,
        "coldOutlet": 294.7104660752987,
        "hotInlet": {
          "molar_flow_mol_s": 14.0,
          "pressure_pa": 100000.0,
          "temperature_k": 303.15
        },
        "coldInlet": {
          "molar_flow_mol_s": 50.0,
          "pressure_pa": 100000.0,
          "temperature_k": 293.15
        },
        "flow": {
          "flow_revision": 1,
          "package": {
            "id": "h3-pure-water-lite-v1",
            "payload_fingerprint": "sha256:1d175d0a82c4d6f586219a32c1bdbad46555356e5d2d50eab3e6bd7a0cec0c26",
            "version": "1"
          },
          "run_id": "review",
          "scope": "request",
          "sequence": 1,
          "snapshot_id": "h3-linked-review-pair-rev-1-seq-1"
        },
        "qualification": {
          "id": "h3-pure-water",
          "phase_reference": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C7732185&Type=ANTOINE",
          "revision": 1
        }
      }
    }
  ],
  "scenarios": {
    "phase": {
      "code": "equipment.h3.flow.not_single_liquid",
      "streams": [
        {
          "side": "cold",
          "stream": {
            "creation_id": "00000000-0000-4000-8000-0000000001f7",
            "stream_id": "cold-feed-50"
          }
        }
      ],
      "projectSha": "d0d5459651af2c1542f616dad948b64eb2ba9e22eb910e14db4e471136eed64f"
    },
    "source": {
      "code": "equipment.h3.flow.properties_unqualified",
      "streams": [
        {
          "side": "cold",
          "stream": {
            "creation_id": "00000000-0000-4000-8000-0000000001f7",
            "stream_id": "cold-feed-50"
          }
        }
      ],
      "projectSha": "64cf692d373c89ecbcf52f3a4bc8bf71710aabf8570bc7b923609a4000a30c81"
    },
    "flow": {
      "code": "solver.step.execution",
      "streams": [],
      "projectSha": "51273427a34c72e033f09238ff1d7774b5fd9e78466947d5f3a9d2c01e41d4ea"
    }
  },
  "baseCommit": "d25dec51"
};
