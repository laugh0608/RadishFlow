// Saved-project review samples captured from CLI v13.
const reviewEvidence = {
  "benchmark": {
    "filename": "h3-benchmark.rfproj.json",
    "file": {
      "document_id": "h3-review-benchmark",
      "project_schema_version": 4,
      "flow_revision": 0,
      "file_sha256": "a9ef874c6185a498340eba4bfe4d8ec90d6a4b971d0f84872bc4cec96090a5ad",
      "read_only": false,
      "equipment_task_count": 2
    },
    "cases": [
      {
        "record": {
          "case": {
            "cold": {
              "inlet": {
                "molar_flow_mol_s": 40.0,
                "pressure_pa": 100000.0,
                "temperature_k": 293.15
              },
              "properties": {
                "density_kg_per_m3": 997.047013,
                "dynamic_viscosity_pa_s": 0.000889996774,
                "liquid_domain": {
                  "max_pressure_pa": 120000.0,
                  "max_temperature_k": 303.15,
                  "min_pressure_pa": 50000.0,
                  "min_temperature_k": 293.15
                },
                "molar_heat_capacity_j_per_mol_k": 75.3299237376304,
                "molar_mass_kg_per_mol": 0.01801528,
                "source_id": "nist-water-table-8",
                "thermal_conductivity_w_per_m_k": 0.607198237
              }
            },
            "hot": {
              "inlet": {
                "molar_flow_mol_s": 14.0,
                "pressure_pa": 100000.0,
                "temperature_k": 303.15
              },
              "properties": {
                "density_kg_per_m3": 997.047013,
                "dynamic_viscosity_pa_s": 0.000889996774,
                "liquid_domain": {
                  "max_pressure_pa": 120000.0,
                  "max_temperature_k": 303.15,
                  "min_pressure_pa": 50000.0,
                  "min_temperature_k": 293.15
                },
                "molar_heat_capacity_j_per_mol_k": 75.3299237376304,
                "molar_mass_kg_per_mol": 0.01801528,
                "source_id": "nist-water-table-8",
                "thermal_conductivity_w_per_m_k": 0.607198237
              }
            },
            "limits": {
              "max_cold_pressure_drop_pa": 25000.0,
              "max_hot_pressure_drop_pa": 11000.0,
              "min_heat_duty_w": 5500.0
            },
            "purpose": "Compare heat duty and straight-run pressure drops"
          },
          "geometry": {
            "active_length_m": 4.85,
            "inner_tube_inner_diameter_m": 0.01446,
            "inner_tube_outer_diameter_m": 0.01588,
            "outer_tube_inner_diameter_m": 0.02676,
            "source_id": "van-zyl-test-section-3",
            "wall_thermal_conductivity_w_per_m_k": 385.0
          },
          "object": {
            "case_id": "h3-water",
            "case_name": "Water",
            "kind": "concentric_tube_rating",
            "method": {
              "id": "radishflow.heat-exchanger.concentric-tube-rating",
              "version": 1
            },
            "name": "Concentric tube benchmark",
            "revisions": {
              "case": 11,
              "geometry": 7,
              "task": 5
            },
            "tag": "E-301",
            "target": {
              "case": {
                "case": {
                  "id": "h3-water"
                },
                "task": {
                  "id": "h3-rating"
                }
              }
            },
            "task_id": "h3-rating"
          },
          "sources": [
            {
              "data": {
                "adoption_note": "Water at 298.15 K and 0.1 MPa",
                "applicability": {
                  "fluid_system": "Water",
                  "operating_conditions": {
                    "pressure_pa": 100000.0,
                    "temperature_k": 298.15
                  },
                  "phase": "liquid",
                  "scope": "Frozen liquid properties"
                },
                "kind": "literature",
                "locator": "Table 8",
                "observations": [],
                "publication_version": "",
                "reference": "https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=832373",
                "review": "reviewed",
                "title": "NIST water properties, table 8"
              },
              "id": "nist-water-table-8",
              "revision": 3
            },
            {
              "data": {
                "adoption_note": "Concentric-tube benchmark dimensions",
                "applicability": {
                  "fluid_system": "Water",
                  "phase": "liquid",
                  "scope": "Test section geometry"
                },
                "kind": "literature",
                "locator": "Table 5",
                "observations": [],
                "publication_version": "",
                "reference": "Van Zyl et al.",
                "review": "reviewed",
                "title": "Van Zyl et al., test section 3"
              },
              "id": "van-zyl-test-section-3",
              "revision": 3
            }
          ]
        },
        "result": {
          "checks": {
            "cold_pressure_drop": "passed",
            "heat_duty": "passed",
            "hot_pressure_drop": "passed"
          },
          "cold": {
            "outlet": {
              "molar_flow_mol_s": 40.0,
              "pressure_pa": 76320.86995952079,
              "temperature_k": 294.9825080401212
            },
            "passage": {
              "darcy_friction_factor": 0.02708123473933345,
              "film_heat_transfer_coefficient_w_per_m2_k": 8237.840481168327,
              "heat_capacity_rate_w_per_k": 3013.196949505216,
              "nusselt_number": 147.60863746564436,
              "prandtl_number": 6.128926904072389,
              "pressure_drop_pa": 23679.13004047922,
              "reynolds_number": 24177.171879271125
            },
            "sensible_energy_residual_w": 2.9103830456733704e-11
          },
          "cold_inlet_approach_k": 4.764262742510766,
          "effectiveness": 0.5235737257489231,
          "heat_duty_w": 5521.707636437074,
          "hot": {
            "outlet": {
              "molar_flow_mol_s": 14.0,
              "pressure_pa": 90200.10303802621,
              "temperature_k": 297.91426274251074
            },
            "passage": {
              "darcy_friction_factor": 0.024700658223291948,
              "film_heat_transfer_coefficient_w_per_m2_k": 7161.7318978313415,
              "heat_capacity_rate_w_per_k": 1054.6189323268254,
              "nusselt_number": 170.55162043008568,
              "prandtl_number": 6.128926904072389,
              "pressure_drop_pa": 9799.896961973793,
              "reynolds_number": 24952.981544000162
            },
            "sensible_energy_residual_w": -2.7284841053187847e-12
          },
          "hot_inlet_approach_k": 8.16749195987876,
          "inner_heat_transfer_area_m2": 0.22032303438890577,
          "lmtd_heat_duty_residual_w": 6.366462912410498e-12,
          "lmtd_k": 6.3137453819237495,
          "origin": {
            "case_id": "h3-water",
            "document_id": "h3-review-benchmark",
            "file_sha256": "a9ef874c6185a498340eba4bfe4d8ec90d6a4b971d0f84872bc4cec96090a5ad",
            "method": {
              "id": "radishflow.heat-exchanger.concentric-tube-rating",
              "version": 1
            },
            "revisions": {
              "case": 11,
              "geometry": 7,
              "task": 5
            },
            "source_ids": {
              "cold_properties": "nist-water-table-8",
              "geometry": "van-zyl-test-section-3",
              "hot_properties": "nist-water-table-8"
            },
            "step_id": "rate",
            "task_id": "h3-rating"
          },
          "outer_heat_transfer_area_m2": 0.24195918299417862,
          "overall_heat_transfer_coefficient_outer_w_per_m2_k": 3614.4667554771145,
          "overall_sensible_energy_residual_w": 2.637534635141492e-11,
          "status": "current",
          "ua_w_per_k": 874.5534231148622
        }
      },
      {
        "record": {
          "case": {
            "cold": {
              "inlet": {
                "molar_flow_mol_s": 50.0,
                "pressure_pa": 100000.0,
                "temperature_k": 293.15
              },
              "properties": {
                "density_kg_per_m3": 997.047013,
                "dynamic_viscosity_pa_s": 0.000889996774,
                "liquid_domain": {
                  "max_pressure_pa": 120000.0,
                  "max_temperature_k": 303.15,
                  "min_pressure_pa": 50000.0,
                  "min_temperature_k": 293.15
                },
                "molar_heat_capacity_j_per_mol_k": 75.3299237376304,
                "molar_mass_kg_per_mol": 0.01801528,
                "source_id": "nist-water-table-8",
                "thermal_conductivity_w_per_m_k": 0.607198237
              }
            },
            "hot": {
              "inlet": {
                "molar_flow_mol_s": 14.0,
                "pressure_pa": 100000.0,
                "temperature_k": 303.15
              },
              "properties": {
                "density_kg_per_m3": 997.047013,
                "dynamic_viscosity_pa_s": 0.000889996774,
                "liquid_domain": {
                  "max_pressure_pa": 120000.0,
                  "max_temperature_k": 303.15,
                  "min_pressure_pa": 50000.0,
                  "min_temperature_k": 293.15
                },
                "molar_heat_capacity_j_per_mol_k": 75.3299237376304,
                "molar_mass_kg_per_mol": 0.01801528,
                "source_id": "nist-water-table-8",
                "thermal_conductivity_w_per_m_k": 0.607198237
              }
            },
            "limits": {
              "max_cold_pressure_drop_pa": 25000.0,
              "max_hot_pressure_drop_pa": 11000.0,
              "min_heat_duty_w": 5500.0
            },
            "purpose": "Compare heat duty and straight-run pressure drops"
          },
          "geometry": {
            "active_length_m": 4.85,
            "inner_tube_inner_diameter_m": 0.01446,
            "inner_tube_outer_diameter_m": 0.01588,
            "outer_tube_inner_diameter_m": 0.02676,
            "source_id": "van-zyl-test-section-3",
            "wall_thermal_conductivity_w_per_m_k": 385.0
          },
          "object": {
            "case_id": "h3-water-50",
            "case_name": "Water",
            "kind": "concentric_tube_rating",
            "method": {
              "id": "radishflow.heat-exchanger.concentric-tube-rating",
              "version": 1
            },
            "name": "Concentric tube benchmark",
            "revisions": {
              "case": 11,
              "geometry": 7,
              "task": 5
            },
            "tag": "E-302",
            "target": {
              "case": {
                "case": {
                  "id": "h3-water-50"
                },
                "task": {
                  "id": "h3-rating-50"
                }
              }
            },
            "task_id": "h3-rating-50"
          },
          "sources": [
            {
              "data": {
                "adoption_note": "Water at 298.15 K and 0.1 MPa",
                "applicability": {
                  "fluid_system": "Water",
                  "operating_conditions": {
                    "pressure_pa": 100000.0,
                    "temperature_k": 298.15
                  },
                  "phase": "liquid",
                  "scope": "Frozen liquid properties"
                },
                "kind": "literature",
                "locator": "Table 8",
                "observations": [],
                "publication_version": "",
                "reference": "https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=832373",
                "review": "reviewed",
                "title": "NIST water properties, table 8"
              },
              "id": "nist-water-table-8",
              "revision": 3
            },
            {
              "data": {
                "adoption_note": "Concentric-tube benchmark dimensions",
                "applicability": {
                  "fluid_system": "Water",
                  "phase": "liquid",
                  "scope": "Test section geometry"
                },
                "kind": "literature",
                "locator": "Table 5",
                "observations": [],
                "publication_version": "",
                "reference": "Van Zyl et al.",
                "review": "reviewed",
                "title": "Van Zyl et al., test section 3"
              },
              "id": "van-zyl-test-section-3",
              "revision": 3
            }
          ]
        },
        "result": {
          "checks": {
            "cold_pressure_drop": "failed",
            "heat_duty": "passed",
            "hot_pressure_drop": "passed"
          },
          "cold": {
            "outlet": {
              "molar_flow_mol_s": 50.0,
              "pressure_pa": 65037.45231473646,
              "temperature_k": 294.7104660752987
            },
            "passage": {
              "darcy_friction_factor": 0.025590912080466932,
              "film_heat_transfer_coefficient_w_per_m2_k": 9926.434119260544,
              "heat_capacity_rate_w_per_k": 3766.49618688152,
              "nusselt_number": 177.86547561658145,
              "prandtl_number": 6.128926904072389,
              "pressure_drop_pa": 34962.54768526354,
              "reynolds_number": 30221.464849088916
            },
            "sensible_energy_residual_w": -1.7280399333685637e-11
          },
          "cold_inlet_approach_k": 4.426906873933092,
          "effectiveness": 0.5573093126066926,
          "heat_duty_w": 5877.489522370671,
          "hot": {
            "outlet": {
              "molar_flow_mol_s": 14.0,
              "pressure_pa": 90200.10303802621,
              "temperature_k": 297.57690687393307
            },
            "passage": {
              "darcy_friction_factor": 0.024700658223291948,
              "film_heat_transfer_coefficient_w_per_m2_k": 7161.7318978313415,
              "heat_capacity_rate_w_per_k": 1054.6189323268254,
              "nusselt_number": 170.55162043008568,
              "prandtl_number": 6.128926904072389,
              "pressure_drop_pa": 9799.896961973793,
              "reynolds_number": 24952.981544000162
            },
            "sensible_energy_residual_w": 1.8189894035458565e-11
          },
          "hot_inlet_approach_k": 8.439533924701266,
          "inner_heat_transfer_area_m2": 0.22032303438890577,
          "lmtd_heat_duty_residual_w": -1.2732925824820995e-11,
          "lmtd_k": 6.218948483426788,
          "origin": {
            "case_id": "h3-water-50",
            "document_id": "h3-review-benchmark",
            "file_sha256": "a9ef874c6185a498340eba4bfe4d8ec90d6a4b971d0f84872bc4cec96090a5ad",
            "method": {
              "id": "radishflow.heat-exchanger.concentric-tube-rating",
              "version": 1
            },
            "revisions": {
              "case": 11,
              "geometry": 7,
              "task": 5
            },
            "source_ids": {
              "cold_properties": "nist-water-table-8",
              "geometry": "van-zyl-test-section-3",
              "hot_properties": "nist-water-table-8"
            },
            "step_id": "rate",
            "task_id": "h3-rating-50"
          },
          "outer_heat_transfer_area_m2": 0.24195918299417862,
          "overall_heat_transfer_coefficient_outer_w_per_m2_k": 3906.005000949918,
          "overall_sensible_energy_residual_w": 9.094947017729282e-13,
          "status": "current",
          "ua_w_per_k": 945.0937788010181
        }
      }
    ]
  },
  "missing": {
    "filename": "h3-missing.rfproj.json",
    "file": {
      "document_id": "h3-review-missing",
      "project_schema_version": 3,
      "flow_revision": 0,
      "file_sha256": "3f0eff794f664646eb19b4ec59c800793bc38b1d8950a81a4000c73076aadecf",
      "read_only": false,
      "equipment_task_count": 1
    },
    "cases": [
      {
        "record": {
          "case": {
            "cold": {
              "inlet": {
                "molar_flow_mol_s": 40.0,
                "pressure_pa": 100000.0,
                "temperature_k": 293.15
              },
              "properties": {
                "density_kg_per_m3": 997.047013,
                "dynamic_viscosity_pa_s": 0.000889996774,
                "liquid_domain": {
                  "max_pressure_pa": 120000.0,
                  "max_temperature_k": 303.15,
                  "min_pressure_pa": 50000.0,
                  "min_temperature_k": 293.15
                },
                "molar_heat_capacity_j_per_mol_k": 75.3299237376304,
                "molar_mass_kg_per_mol": 0.01801528,
                "source_id": "nist-water-table-8",
                "thermal_conductivity_w_per_m_k": 0.607198237
              }
            },
            "hot": {
              "inlet": {
                "molar_flow_mol_s": 14.0,
                "pressure_pa": 100000.0,
                "temperature_k": 303.15
              },
              "properties": {
                "density_kg_per_m3": 997.047013,
                "dynamic_viscosity_pa_s": 0.000889996774,
                "liquid_domain": {
                  "max_pressure_pa": 120000.0,
                  "max_temperature_k": 303.15,
                  "min_pressure_pa": 50000.0,
                  "min_temperature_k": 293.15
                },
                "molar_heat_capacity_j_per_mol_k": 75.3299237376304,
                "molar_mass_kg_per_mol": 0.01801528,
                "source_id": "nist-water-table-8",
                "thermal_conductivity_w_per_m_k": 0.607198237
              }
            },
            "limits": {
              "max_cold_pressure_drop_pa": 25000.0,
              "max_hot_pressure_drop_pa": 11000.0,
              "min_heat_duty_w": 5500.0
            },
            "purpose": "Compare heat duty and straight-run pressure drops"
          },
          "geometry": {
            "active_length_m": null,
            "inner_tube_inner_diameter_m": 0.01446,
            "inner_tube_outer_diameter_m": 0.01588,
            "outer_tube_inner_diameter_m": 0.02676,
            "source_id": "van-zyl-test-section-3",
            "wall_thermal_conductivity_w_per_m_k": 385.0
          },
          "object": {
            "case_id": "h3-water",
            "case_name": "Water",
            "kind": "concentric_tube_rating",
            "method": {
              "id": "radishflow.heat-exchanger.concentric-tube-rating",
              "version": 1
            },
            "name": "Concentric tube benchmark",
            "revisions": {
              "case": 11,
              "geometry": 7,
              "task": 5
            },
            "tag": "E-301",
            "target": {
              "case": {
                "case": {
                  "id": "h3-water"
                },
                "task": {
                  "id": "h3-rating"
                }
              }
            },
            "task_id": "h3-rating"
          },
          "sources": [
            {
              "data": {
                "adoption_note": "Water at 298.15 K and 0.1 MPa",
                "applicability": {
                  "fluid_system": "Water",
                  "operating_conditions": {
                    "pressure_pa": 100000.0,
                    "temperature_k": 298.15
                  },
                  "phase": "liquid",
                  "scope": "Frozen liquid properties"
                },
                "kind": "literature",
                "locator": "Table 8",
                "observations": [],
                "publication_version": "",
                "reference": "https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=832373",
                "review": "reviewed",
                "title": "NIST water properties, table 8"
              },
              "id": "nist-water-table-8",
              "revision": 3
            },
            {
              "data": {
                "adoption_note": "Concentric-tube benchmark dimensions",
                "applicability": {
                  "fluid_system": "Water",
                  "phase": "liquid",
                  "scope": "Test section geometry"
                },
                "kind": "literature",
                "locator": "Table 5",
                "observations": [],
                "publication_version": "",
                "reference": "Van Zyl et al.",
                "review": "reviewed",
                "title": "Van Zyl et al., test section 3"
              },
              "id": "van-zyl-test-section-3",
              "revision": 3
            }
          ]
        },
        "result": {
          "diagnostic": {
            "code": "unit.heat_exchanger.concentric_tube.missing_input",
            "message": "geometry.activeLengthM: required for rating"
          },
          "origin": {
            "case_id": "h3-water",
            "document_id": "h3-review-missing",
            "file_sha256": "3f0eff794f664646eb19b4ec59c800793bc38b1d8950a81a4000c73076aadecf",
            "method": {
              "id": "radishflow.heat-exchanger.concentric-tube-rating",
              "version": 1
            },
            "revisions": {
              "case": 11,
              "geometry": 7,
              "task": 5
            },
            "source_ids": {
              "cold_properties": "nist-water-table-8",
              "geometry": "van-zyl-test-section-3",
              "hot_properties": "nist-water-table-8"
            },
            "step_id": "rate",
            "task_id": "h3-rating"
          },
          "status": "failed"
        }
      }
    ]
  },
  "domain": {
    "filename": "h3-domain.rfproj.json",
    "file": {
      "document_id": "h3-review-domain",
      "project_schema_version": 3,
      "flow_revision": 0,
      "file_sha256": "926a51f47277b440e6e1f51640cc4c4304a65ad6da225b08d32e96ad6efc1d11",
      "read_only": false,
      "equipment_task_count": 1
    },
    "cases": [
      {
        "record": {
          "case": {
            "cold": {
              "inlet": {
                "molar_flow_mol_s": 40.0,
                "pressure_pa": 150000.0,
                "temperature_k": 293.15
              },
              "properties": {
                "density_kg_per_m3": 997.047013,
                "dynamic_viscosity_pa_s": 0.000889996774,
                "liquid_domain": {
                  "max_pressure_pa": 120000.0,
                  "max_temperature_k": 303.15,
                  "min_pressure_pa": 50000.0,
                  "min_temperature_k": 293.15
                },
                "molar_heat_capacity_j_per_mol_k": 75.3299237376304,
                "molar_mass_kg_per_mol": 0.01801528,
                "source_id": "nist-water-table-8",
                "thermal_conductivity_w_per_m_k": 0.607198237
              }
            },
            "hot": {
              "inlet": {
                "molar_flow_mol_s": 14.0,
                "pressure_pa": 100000.0,
                "temperature_k": 303.15
              },
              "properties": {
                "density_kg_per_m3": 997.047013,
                "dynamic_viscosity_pa_s": 0.000889996774,
                "liquid_domain": {
                  "max_pressure_pa": 120000.0,
                  "max_temperature_k": 303.15,
                  "min_pressure_pa": 50000.0,
                  "min_temperature_k": 293.15
                },
                "molar_heat_capacity_j_per_mol_k": 75.3299237376304,
                "molar_mass_kg_per_mol": 0.01801528,
                "source_id": "nist-water-table-8",
                "thermal_conductivity_w_per_m_k": 0.607198237
              }
            },
            "limits": {
              "max_cold_pressure_drop_pa": 25000.0,
              "max_hot_pressure_drop_pa": 11000.0,
              "min_heat_duty_w": 5500.0
            },
            "purpose": "Compare heat duty and straight-run pressure drops"
          },
          "geometry": {
            "active_length_m": 4.85,
            "inner_tube_inner_diameter_m": 0.01446,
            "inner_tube_outer_diameter_m": 0.01588,
            "outer_tube_inner_diameter_m": 0.02676,
            "source_id": "van-zyl-test-section-3",
            "wall_thermal_conductivity_w_per_m_k": 385.0
          },
          "object": {
            "case_id": "h3-water",
            "case_name": "Water",
            "kind": "concentric_tube_rating",
            "method": {
              "id": "radishflow.heat-exchanger.concentric-tube-rating",
              "version": 1
            },
            "name": "Concentric tube benchmark",
            "revisions": {
              "case": 11,
              "geometry": 7,
              "task": 5
            },
            "tag": "E-301",
            "target": {
              "case": {
                "case": {
                  "id": "h3-water"
                },
                "task": {
                  "id": "h3-rating"
                }
              }
            },
            "task_id": "h3-rating"
          },
          "sources": [
            {
              "data": {
                "adoption_note": "Water at 298.15 K and 0.1 MPa",
                "applicability": {
                  "fluid_system": "Water",
                  "operating_conditions": {
                    "pressure_pa": 100000.0,
                    "temperature_k": 298.15
                  },
                  "phase": "liquid",
                  "scope": "Frozen liquid properties"
                },
                "kind": "literature",
                "locator": "Table 8",
                "observations": [],
                "publication_version": "",
                "reference": "https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=832373",
                "review": "reviewed",
                "title": "NIST water properties, table 8"
              },
              "id": "nist-water-table-8",
              "revision": 3
            },
            {
              "data": {
                "adoption_note": "Concentric-tube benchmark dimensions",
                "applicability": {
                  "fluid_system": "Water",
                  "phase": "liquid",
                  "scope": "Test section geometry"
                },
                "kind": "literature",
                "locator": "Table 5",
                "observations": [],
                "publication_version": "",
                "reference": "Van Zyl et al.",
                "review": "reviewed",
                "title": "Van Zyl et al., test section 3"
              },
              "id": "van-zyl-test-section-3",
              "revision": 3
            }
          ]
        },
        "result": {
          "diagnostic": {
            "code": "unit.heat_exchanger.concentric_tube.inlet_outside_domain",
            "message": "cold annulus inlet at 293.15 K and 150000 Pa is outside the liquid domain"
          },
          "origin": {
            "case_id": "h3-water",
            "document_id": "h3-review-domain",
            "file_sha256": "926a51f47277b440e6e1f51640cc4c4304a65ad6da225b08d32e96ad6efc1d11",
            "method": {
              "id": "radishflow.heat-exchanger.concentric-tube-rating",
              "version": 1
            },
            "revisions": {
              "case": 11,
              "geometry": 7,
              "task": 5
            },
            "source_ids": {
              "cold_properties": "nist-water-table-8",
              "geometry": "van-zyl-test-section-3",
              "hot_properties": "nist-water-table-8"
            },
            "step_id": "rate",
            "task_id": "h3-rating"
          },
          "status": "failed"
        }
      }
    ]
  }
};
