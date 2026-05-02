mod auth_cache;
mod integrity;
mod json;
mod layout;
mod package_cache;
mod project;
mod studio_layout;
mod studio_preferences;

pub use auth_cache::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
    StoredPropertyPackageRecord, StoredPropertyPackageSource,
};
pub use integrity::{
    StoredPropertyPackagePayloadIntegrity, property_package_payload_integrity,
    validate_property_package_payload_integrity,
};
pub use json::{
    auth_cache_index_to_pretty_json, parse_auth_cache_index_json, parse_project_file_json,
    parse_property_package_manifest_json, parse_property_package_payload_json,
    parse_studio_layout_file_json, parse_studio_preferences_file_json, project_file_to_pretty_json,
    property_package_manifest_to_pretty_json, property_package_payload_to_pretty_json,
    read_auth_cache_index, read_project_file, read_property_package_manifest,
    read_property_package_payload, read_studio_layout_file, read_studio_preferences_file,
    studio_layout_file_to_pretty_json, studio_preferences_file_to_pretty_json,
    write_auth_cache_index, write_project_file, write_property_package_manifest,
    write_property_package_payload, write_studio_layout_file, write_studio_preferences_file,
};
pub use json::{option_time_format, time_format};
pub use layout::{
    STORED_AUTH_CACHE_INDEX_FILE_NAME, STORED_AUTH_ROOT_DIR, STORED_PACKAGE_CACHE_ROOT_DIR,
    STORED_PROPERTY_PACKAGE_MANIFEST_FILE_NAME, STORED_PROPERTY_PACKAGE_PAYLOAD_FILE_NAME,
    StoredAuthCacheLayout,
};
pub use package_cache::{
    STORED_PROPERTY_PACKAGE_MANIFEST_KIND, STORED_PROPERTY_PACKAGE_PAYLOAD_KIND,
    STORED_PROPERTY_PACKAGE_SCHEMA_VERSION, StoredAntoineCoefficients, StoredLiquidPhaseModel,
    StoredPropertyPackageClassification, StoredPropertyPackageManifest,
    StoredPropertyPackagePayload, StoredThermoComponent, StoredThermoMethod, StoredVaporPhaseModel,
};
pub use project::{
    DateTimeUtc, STORED_PROJECT_FILE_EXTENSION, StoredDocumentMetadata, StoredProjectDocument,
    StoredProjectFile,
};
pub use studio_layout::{
    STORED_STUDIO_LAYOUT_FILE_KIND, STORED_STUDIO_LAYOUT_FILE_SUFFIX,
    STORED_STUDIO_LAYOUT_SCHEMA_VERSION, StoredStudioLayoutFile, StoredStudioLayoutPanelState,
    StoredStudioLayoutRegionWeight, StoredStudioLayoutStackGroupState,
    StoredStudioWindowLayoutEntry, studio_layout_path_for_project,
};
pub use studio_preferences::{
    STORED_STUDIO_PREFERENCES_FILE_KIND, STORED_STUDIO_PREFERENCES_FILE_NAME,
    STORED_STUDIO_PREFERENCES_SCHEMA_VERSION, StoredStudioPreferencesFile,
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_model::{Component, Flowsheet};
    use rf_types::ComponentId;

    use crate::{
        STORED_PROJECT_FILE_EXTENSION, StoredAuthCacheIndex, StoredAuthCacheLayout,
        StoredCredentialReference, StoredDocumentMetadata, StoredEntitlementCache,
        StoredProjectFile, StoredPropertyPackageManifest, StoredPropertyPackagePayload,
        StoredPropertyPackageRecord, StoredPropertyPackageSource, StoredStudioLayoutFile,
        StoredStudioLayoutPanelState, StoredStudioLayoutRegionWeight,
        StoredStudioLayoutStackGroupState, StoredStudioPreferencesFile,
        StoredStudioWindowLayoutEntry, StoredThermoComponent, auth_cache_index_to_pretty_json,
        parse_auth_cache_index_json, parse_project_file_json, parse_property_package_manifest_json,
        parse_property_package_payload_json, parse_studio_layout_file_json,
        parse_studio_preferences_file_json, project_file_to_pretty_json,
        property_package_manifest_to_pretty_json, property_package_payload_to_pretty_json,
        read_auth_cache_index, read_project_file, read_property_package_manifest,
        read_property_package_payload, read_studio_layout_file, read_studio_preferences_file,
        studio_layout_file_to_pretty_json, studio_layout_path_for_project,
        studio_preferences_file_to_pretty_json, write_auth_cache_index, write_project_file,
        write_property_package_manifest, write_property_package_payload, write_studio_layout_file,
        write_studio_preferences_file,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
    }

    #[test]
    fn project_file_keeps_document_payload_separate_from_auth_cache() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_component(Component::new(ComponentId::new("methane"), "Methane"))
            .expect("expected component");
        let metadata = StoredDocumentMetadata::new("doc-1", "Demo Project", timestamp(10));
        let project = StoredProjectFile::new(flowsheet, metadata);
        let auth_cache = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );

        assert_eq!(project.kind, "radishflow.project-file");
        assert_eq!(project.schema_version, 1);
        assert_eq!(project.document.metadata.document_id, "doc-1");
        assert_eq!(STORED_PROJECT_FILE_EXTENSION, ".rfproj.json");
        assert_eq!(auth_cache.kind, "radishflow.auth-cache-index");
        assert_eq!(project.document.revision, 0);
        assert_eq!(auth_cache.schema_version, 1);
        assert!(auth_cache.entitlement.is_none());
        assert_eq!(
            auth_cache.index_relative_path(),
            PathBuf::from("auth")
                .join("https_id.radish.local")
                .join("user-123")
                .join("index.json")
        );
    }

    #[test]
    fn property_package_record_reports_expiration_from_cached_metadata() {
        let mut record = StoredPropertyPackageRecord::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:test",
            1024,
            timestamp(100),
        );
        record.expires_at = Some(timestamp(200));

        assert!(!record.is_expired_at(timestamp(150)));
        assert!(record.is_expired_at(timestamp(250)));
        assert_eq!(
            record.manifest_relative_path,
            StoredAuthCacheLayout::package_manifest_relative_path(
                "binary-hydrocarbon-lite-v1",
                "2026.03.1"
            )
        );
        assert_eq!(
            record.payload_relative_path,
            Some(StoredAuthCacheLayout::package_payload_relative_path(
                "binary-hydrocarbon-lite-v1",
                "2026.03.1"
            ))
        );
    }

    #[test]
    fn remote_evaluation_packages_do_not_claim_local_payload_paths() {
        let record = StoredPropertyPackageRecord::new(
            "premium-eos-v1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteEvaluationService,
            "sha256:test",
            0,
            timestamp(300),
        );

        assert!(record.payload_relative_path.is_none());
        assert_eq!(
            record.manifest_relative_path,
            PathBuf::from("packages")
                .join("premium-eos-v1")
                .join("2026.03.1")
                .join("manifest.json")
        );
    }

    #[test]
    fn project_file_round_trips_as_camel_case_json() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_component(Component::new(ComponentId::new("methane"), "Methane"))
            .expect("expected component");
        let project = StoredProjectFile::new(
            flowsheet,
            StoredDocumentMetadata::new("doc-1", "Demo Project", timestamp(10)),
        );

        let json = project_file_to_pretty_json(&project).expect("expected project json");
        let round_trip = parse_project_file_json(&json).expect("expected project parse");

        assert_eq!(round_trip, project);
        assert!(json.contains("\"kind\": \"radishflow.project-file\""));
        assert!(json.contains("\"schemaVersion\": 1"));
        assert!(json.contains("\"documentId\": \"doc-1\""));
        assert!(json.contains("\"createdAt\": \"1970-01-01T00:00:10Z\""));
    }

    #[test]
    fn auth_cache_index_round_trips_as_camel_case_json() {
        let mut auth_cache = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        auth_cache.entitlement = Some(StoredEntitlementCache {
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            synced_at: timestamp(100),
            issued_at: timestamp(90),
            expires_at: timestamp(200),
            offline_lease_expires_at: Some(timestamp(300)),
            feature_keys: BTreeSet::from([
                "desktop-login".to_string(),
                "local-thermo-packages".to_string(),
            ]),
            allowed_package_ids: BTreeSet::from(["pkg-1".to_string()]),
        });
        auth_cache
            .property_packages
            .push(StoredPropertyPackageRecord::new(
                "pkg-1",
                "2026.03.1",
                StoredPropertyPackageSource::RemoteDerivedPackage,
                "sha256:test",
                1024,
                timestamp(110),
            ));
        auth_cache.last_synced_at = Some(timestamp(111));

        let json = auth_cache_index_to_pretty_json(&auth_cache).expect("expected auth cache json");
        let round_trip =
            parse_auth_cache_index_json(&json).expect("expected auth cache parse round trip");

        assert_eq!(round_trip, auth_cache);
        assert!(json.contains("\"kind\": \"radishflow.auth-cache-index\""));
        assert!(json.contains("\"authorityUrl\": \"https://id.radish.local\""));
        assert!(
            json.contains("\"manifestRelativePath\": \"packages/pkg-1/2026.03.1/manifest.json\"")
        );
        assert!(json.contains("\"downloadedAt\": \"1970-01-01T00:01:50Z\""));
    }

    #[test]
    fn property_package_manifest_round_trips_as_camel_case_json() {
        let mut manifest = StoredPropertyPackageManifest::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            vec![ComponentId::new("methane"), ComponentId::new("ethane")],
        );
        manifest.hash = "sha256:pkg-1".to_string();
        manifest.size_bytes = 1024;
        manifest.expires_at = Some(timestamp(700));

        let json =
            property_package_manifest_to_pretty_json(&manifest).expect("expected manifest json");
        let round_trip = parse_property_package_manifest_json(&json)
            .expect("expected manifest parse round trip");

        assert_eq!(round_trip, manifest);
        assert!(json.contains("\"kind\": \"radishflow.property-package-manifest\""));
        assert!(json.contains("\"schemaVersion\": 1"));
        assert!(json.contains("\"leaseRequired\": true"));
        assert!(json.contains("\"componentIds\": ["));
    }

    #[test]
    fn property_package_payload_round_trips_as_camel_case_json() {
        let mut methane = StoredThermoComponent::new(ComponentId::new("methane"), "Methane");
        methane.liquid_heat_capacity_j_per_mol_k = Some(35.0);
        let mut payload = StoredPropertyPackagePayload::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            vec![methane],
        );
        payload.method = Default::default();

        let json =
            property_package_payload_to_pretty_json(&payload).expect("expected payload json");
        let round_trip =
            parse_property_package_payload_json(&json).expect("expected payload parse round trip");

        assert_eq!(round_trip, payload);
        assert!(json.contains("\"kind\": \"radishflow.property-package-payload\""));
        assert!(json.contains("\"schemaVersion\": 1"));
        assert!(json.contains("\"liquidPhaseModel\": \"ideal-solution\""));
    }

    #[test]
    fn studio_layout_file_round_trips_as_camel_case_json() {
        let layout = StoredStudioLayoutFile::new(vec![StoredStudioWindowLayoutEntry {
            layout_key: "studio.window.owner.slot-1".to_string(),
            center_area: "canvas".to_string(),
            panels: vec![StoredStudioLayoutPanelState {
                area_id: "commands".to_string(),
                dock_region: "left-sidebar".to_string(),
                stack_group: 10,
                order: 10,
                visible: true,
                collapsed: false,
            }],
            stack_groups: vec![StoredStudioLayoutStackGroupState {
                dock_region: "left-sidebar".to_string(),
                stack_group: 10,
                active_area_id: "commands".to_string(),
            }],
            region_weights: vec![StoredStudioLayoutRegionWeight {
                dock_region: "center-stage".to_string(),
                weight: 52,
            }],
        }]);

        let json = studio_layout_file_to_pretty_json(&layout).expect("expected layout json");
        let round_trip =
            parse_studio_layout_file_json(&json).expect("expected layout parse round trip");

        assert_eq!(round_trip, layout);
        assert!(json.contains("\"kind\": \"radishflow.studio-layout-file\""));
        assert!(json.contains("\"layoutKey\": \"studio.window.owner.slot-1\""));
        assert!(json.contains("\"centerArea\": \"canvas\""));
        assert!(json.contains("\"stackGroups\""));
    }

    #[test]
    fn studio_preferences_file_round_trips_as_camel_case_json() {
        let preferences = StoredStudioPreferencesFile::new(vec![
            "D:\\Code\\RadishFlow\\examples\\flowsheets\\feed-heater-flash.rfproj.json".to_string(),
            "D:\\Code\\RadishFlow\\examples\\flowsheets\\feed-valve-flash.rfproj.json".to_string(),
        ]);

        let json = studio_preferences_file_to_pretty_json(&preferences)
            .expect("expected preferences json");
        let round_trip = parse_studio_preferences_file_json(&json)
            .expect("expected preferences parse round trip");

        assert_eq!(round_trip, preferences);
        assert!(json.contains("\"kind\": \"radishflow.studio-preferences-file\""));
        assert!(json.contains("\"schemaVersion\": 1"));
        assert!(json.contains("\"recentProjectPaths\""));
    }

    #[test]
    fn studio_preferences_file_rejects_duplicate_recent_project_paths() {
        let preferences = StoredStudioPreferencesFile::new(vec!["demo.rfproj.json".to_string(); 2]);

        let error = preferences
            .validate()
            .expect_err("expected duplicate path error");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(
            error
                .message()
                .contains("duplicate recent project path `demo.rfproj.json`")
        );
    }

    #[test]
    fn parse_rejects_wrong_project_file_kind() {
        let json = r#"{
  "kind": "wrong-kind",
  "schemaVersion": 1,
  "document": {
    "revision": 0,
    "flowsheet": {
      "name": "demo",
      "components": {},
      "streams": {},
      "units": {}
    },
    "metadata": {
      "documentId": "doc-1",
      "title": "Demo Project",
      "schemaVersion": 1,
      "createdAt": "1970-01-01T00:00:10Z",
      "updatedAt": "1970-01-01T00:00:10Z"
    }
  }
}"#;

        let error = parse_project_file_json(json).expect_err("expected wrong kind error");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(
            error
                .message()
                .contains("unsupported stored project file kind")
        );
    }

    #[test]
    fn parse_rejects_newer_project_file_schema_with_migration_hint() {
        let json = r#"{
  "kind": "radishflow.project-file",
  "schemaVersion": 2,
  "document": {
    "revision": 0,
    "flowsheet": {
      "name": "demo",
      "components": {},
      "streams": {},
      "units": {}
    },
    "metadata": {
      "documentId": "doc-1",
      "title": "Demo Project",
      "schemaVersion": 1,
      "createdAt": "1970-01-01T00:00:10Z",
      "updatedAt": "1970-01-01T00:00:10Z"
    }
  }
}"#;

        let error = parse_project_file_json(json).expect_err("expected newer schema error");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(error.message().contains("newer than supported version"));
        assert!(error.message().contains("add a migration in rf-store"));
    }

    #[test]
    fn parse_rejects_auth_cache_index_without_schema_version() {
        let json = r#"{
  "kind": "radishflow.auth-cache-index",
  "schemaVersion": 0,
  "authorityUrl": "https://id.radish.local",
  "subjectId": "user-123",
  "credential": {
    "service": "radishflow-studio",
    "account": "user-123-primary"
  },
  "propertyPackages": []
}"#;

        let error = parse_auth_cache_index_json(json).expect_err("expected older schema error");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(
            error
                .message()
                .contains("missing required field `schemaVersion`")
        );
    }

    #[test]
    fn write_project_file_creates_parent_directories_and_round_trips() {
        let root = unique_temp_path("project-write");
        let path = root.join("nested").join("demo.rfproj.json");
        let project = StoredProjectFile::new(
            Flowsheet::new("demo"),
            StoredDocumentMetadata::new("doc-1", "Demo Project", timestamp(10)),
        );

        write_project_file(&path, &project).expect("expected project file write");
        let loaded = read_project_file(&path).expect("expected project file read");

        assert_eq!(loaded, project);
        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn write_project_file_replaces_existing_file_from_staged_write() {
        let root = unique_temp_path("project-replace");
        let path = root.join("demo.rfproj.json");
        let first = StoredProjectFile::new(
            Flowsheet::new("first"),
            StoredDocumentMetadata::new("doc-1", "First Project", timestamp(10)),
        );
        let second = StoredProjectFile::new(
            Flowsheet::new("second"),
            StoredDocumentMetadata::new("doc-1", "Second Project", timestamp(20)),
        );

        write_project_file(&path, &first).expect("expected first project file write");
        write_project_file(&path, &second).expect("expected replacement project file write");

        let loaded = read_project_file(&path).expect("expected replaced project file read");
        assert_eq!(loaded, second);
        assert!(staged_siblings_for(&path).is_empty());
        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn write_project_file_rejects_directory_target_without_staged_leftovers() {
        let root = unique_temp_path("project-directory-target");
        let path = root.join("demo.rfproj.json");
        fs::create_dir_all(&path).expect("expected directory target");
        let project = StoredProjectFile::new(
            Flowsheet::new("demo"),
            StoredDocumentMetadata::new("doc-1", "Demo Project", timestamp(10)),
        );

        let error = write_project_file(&path, &project).expect_err("expected write error");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(
            error
                .message()
                .contains("target path exists and is not a file")
        );
        assert!(path.is_dir());
        assert!(staged_siblings_for(&path).is_empty());
        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn studio_layout_path_uses_project_sidecar_suffix() {
        let project = PathBuf::from("demo").join("sample.rfproj.json");
        let sidecar = studio_layout_path_for_project(&project);

        assert_eq!(
            sidecar,
            PathBuf::from("demo").join("sample.rfstudio-layout.json")
        );
    }

    #[test]
    fn write_auth_cache_index_creates_parent_directories_and_round_trips() {
        let root = unique_temp_path("auth-cache-write");
        let path = root.join("auth").join("tenant").join("index.json");
        let auth_cache = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );

        write_auth_cache_index(&path, &auth_cache).expect("expected auth cache write");
        let loaded = read_auth_cache_index(&path).expect("expected auth cache read");

        assert_eq!(loaded, auth_cache);
        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    fn staged_siblings_for(path: &Path) -> Vec<PathBuf> {
        let Some(directory) = path.parent() else {
            return Vec::new();
        };
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let prefix = format!(".{file_name}.");
        fs::read_dir(directory)
            .expect("expected readable temp directory")
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with(&prefix))
                    .unwrap_or(false)
            })
            .collect()
    }

    #[test]
    fn write_studio_layout_file_creates_parent_directories_and_round_trips() {
        let root = unique_temp_path("studio-layout-write");
        let path = root.join("layout").join("sample.rfstudio-layout.json");
        let layout = StoredStudioLayoutFile::new(vec![StoredStudioWindowLayoutEntry {
            layout_key: "studio.window.owner.slot-1".to_string(),
            center_area: "canvas".to_string(),
            panels: vec![StoredStudioLayoutPanelState {
                area_id: "runtime".to_string(),
                dock_region: "right-sidebar".to_string(),
                stack_group: 10,
                order: 30,
                visible: false,
                collapsed: true,
            }],
            stack_groups: vec![StoredStudioLayoutStackGroupState {
                dock_region: "right-sidebar".to_string(),
                stack_group: 10,
                active_area_id: "runtime".to_string(),
            }],
            region_weights: vec![StoredStudioLayoutRegionWeight {
                dock_region: "right-sidebar".to_string(),
                weight: 31,
            }],
        }]);

        write_studio_layout_file(&path, &layout).expect("expected studio layout write");
        let loaded = read_studio_layout_file(&path).expect("expected studio layout read");

        assert_eq!(loaded, layout);
        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn write_studio_preferences_file_creates_parent_directories_and_round_trips() {
        let root = unique_temp_path("studio-preferences-write");
        let path = root
            .join("RadishFlow")
            .join("Studio")
            .join("preferences.rfstudio-preferences.json");
        let preferences = StoredStudioPreferencesFile::new(vec![
            "demo-a.rfproj.json".to_string(),
            "demo-b.rfproj.json".to_string(),
        ]);

        write_studio_preferences_file(&path, &preferences)
            .expect("expected studio preferences write");
        let loaded = read_studio_preferences_file(&path).expect("expected studio preferences read");

        assert_eq!(loaded, preferences);
        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn write_property_package_assets_create_parent_directories_and_round_trip() {
        let root = unique_temp_path("package-cache-write");
        let manifest_path = root
            .join("packages")
            .join("pkg-1")
            .join("2026.03.1")
            .join("manifest.json");
        let payload_path = root
            .join("packages")
            .join("pkg-1")
            .join("2026.03.1")
            .join("payload.rfpkg");
        let manifest = StoredPropertyPackageManifest::new(
            "pkg-1",
            "2026.03.1",
            StoredPropertyPackageSource::LocalBundled,
            vec![ComponentId::new("methane")],
        );
        let payload = StoredPropertyPackagePayload::new(
            "pkg-1",
            "2026.03.1",
            vec![StoredThermoComponent::new(
                ComponentId::new("methane"),
                "Methane",
            )],
        );

        write_property_package_manifest(&manifest_path, &manifest)
            .expect("expected manifest write");
        write_property_package_payload(&payload_path, &payload).expect("expected payload write");

        let loaded_manifest =
            read_property_package_manifest(&manifest_path).expect("expected manifest read");
        let loaded_payload =
            read_property_package_payload(&payload_path).expect("expected payload read");

        assert_eq!(loaded_manifest, manifest);
        assert_eq!(loaded_payload, payload);
        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }
}
