//! Existing codecs must not downsave the new physical model or silently drop its parameter.
use super::*;
use rf_model::{Flowsheet, UnitNode};
use std::time::UNIX_EPOCH;
#[test]
fn all_official_legacy_writers_reject_splitter_and_fraction_on_other_models() {
    for (kind, fraction) in [
        ("splitter", None),
        ("splitter", Some(0.6)),
        ("heater", Some(0.6)),
    ] {
        let mut flowsheet = Flowsheet::new("capability guard");
        let mut unit = UnitNode::new("unit", "Unit", kind, vec![]);
        unit.parameters.outlet_a_split_fraction = fraction;
        flowsheet.insert_unit(unit).unwrap();
        let mut file = StoredProjectFile::new(
            flowsheet.clone(),
            StoredDocumentMetadata::new("id", "Guard", UNIX_EPOCH),
        );
        for version in [1, 2] {
            file.schema_version = version;
            let error = project_file_to_pretty_json(&file).unwrap_err();
            assert_eq!(
                error.context().diagnostic_code(),
                Some("store.project.unsupported_flow_capability")
            );
            assert_eq!(
                error.context().related_unit_ids(),
                &[rf_types::UnitId::new("unit")]
            );
            assert!(serde_json::to_string(&file).is_err());
            assert_eq!(file.document.flowsheet, flowsheet);
        }
        let v3 = project_v3::ProjectV3Read::Editable(project_v3::StoredProjectV3 {
            document: file.document.clone(),
            display_units: Default::default(),
            engineering: None,
        });
        assert!(project_v3::to_pretty_json(&v3).is_err());
    }
}
