use super::*;

#[test]
fn headless_output_failure_is_not_reported_as_success() {
    struct BrokenOutput;
    impl std::io::Write for BrokenOutput {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut errors = Vec::new();
    assert_eq!(run_cli(&[], &mut BrokenOutput, &mut errors), 5);
    assert!(String::from_utf8(errors).unwrap().contains("output failed"));
}

#[test]
fn headless_flush_failure_is_not_reported_as_success() {
    struct FailedFlush(Vec<u8>);
    impl std::io::Write for FailedFlush {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::from(std::io::ErrorKind::Other))
        }
    }
    let mut output = FailedFlush(Vec::new());
    assert_eq!(run_cli(&["--help".into()], &mut output, &mut Vec::new()), 5);
    assert!(!output.0.is_empty());
}

#[test]
fn headless_protocol_rejects_unknown_nested_fields_and_value_coercion() {
    for json in [
        r#"{"target":{"object":{"unit":"a"},"section":"inputs","field":"name","typo":0},"value":"b"}"#,
        r#"{"target":{"object":{"unit":"a"},"section":"inputs","field":"name"},"value":true}"#,
        r#"{"target":{"object":{"unit":"a"},"section":"results","field":{"phase_mole_fraction":{"phase":"vapor","component":"a","extra":1}}},"value":0}"#,
        r#"{"target":{"object":{"unit":"a"},"section":"inputs","field":"temperature"},"value":1e999}"#,
    ] {
        assert!(
            serde_json::from_str::<protocol::Write>(json).is_err(),
            "accepted {json}"
        );
    }
}

#[test]
fn headless_versions_strictly_reject_mixed_duplicate_and_unknown_fields() {
    let example =
        include_str!("../../../../examples/automation/build-feed-heater-flash.request.json");
    assert!(request::parse(example.as_bytes()).is_ok());
    for invalid in [
        example.replacen(
            "\"schema_version\": 2",
            "\"schema_version\": 2, \"schema_version\": 2",
            1,
        ),
        example.replacen("\"id\": \"feed\"", "\"id\": \"feed\", \"id\": \"other\"", 1),
        example.replacen(
            "\"unit_kind\": \"feed\"",
            "\"unit_kind\": \"feed\", \"unit_kind\": \"heater\"",
            1,
        ),
        example.replacen(
            "\"unit_kind\": \"feed\"",
            "\"unit_kind\": \"feed\", \"extra\": true",
            1,
        ),
        example.replacen(
            "\"step\": \"feed\"",
            "\"step\": \"feed\", \"id\": \"feed-1\"",
            1,
        ),
        example.replacen(
            "\"port\": \"outlet\"",
            "\"port\": \"outlet\", \"extra\": true",
            1,
        ),
        example.replacen("\"steps\":", "\"writes\": [], \"steps\":", 1),
        example.replacen("\"schema_version\": 2", "\"schema_version\": 1", 1),
    ] {
        assert!(
            request::parse(invalid.as_bytes()).is_err(),
            "accepted {invalid}"
        );
    }
}

#[test]
fn headless_v2_creation_preserves_application_document_and_history_for_all_kinds() {
    let cache = rf_store::StoredAuthCacheIndex::new(
        "https://example.invalid",
        "headless-actions",
        rf_store::StoredCredentialReference::new("test", "headless-actions"),
    );
    let context = StudioAppAuthCacheContext::new(Path::new("unused"), &cache);
    for (wire_kind, kind) in [
        ("feed", rf_ui::BuiltinUnitKind::Feed),
        ("heater", rf_ui::BuiltinUnitKind::Heater),
        ("cooler", rf_ui::BuiltinUnitKind::Cooler),
        ("valve", rf_ui::BuiltinUnitKind::Valve),
        ("mixer", rf_ui::BuiltinUnitKind::Mixer),
        ("flash_drum", rf_ui::BuiltinUnitKind::FlashDrum),
    ] {
        let mut app = AppState::new(rf_ui::FlowsheetDocument::new(
            rf_model::Flowsheet::new("Headless"),
            rf_ui::DocumentMetadata::new("headless", "Headless", SystemTime::UNIX_EPOCH),
        ));
        let mut direct = app.clone();
        let steps: Vec<workflow::Step> = serde_json::from_value(serde_json::json!([
            {"id":"created", "action":{"kind":"create_unit","unit_kind":wire_kind}}
        ]))
        .unwrap();
        let mut response = Response::default();
        workflow::execute(
            &mut app,
            &context,
            steps,
            &mut workflow::Bindings::default(),
            &mut response,
        )
        .unwrap();
        // Use the transaction's recorded time to compare the full document/history exactly.
        let changed_at = app.workspace.document.metadata.updated_at;
        let expected = direct.create_builtin_unit(kind, changed_at).unwrap();
        assert_eq!(app.workspace.document, direct.workspace.document);
        assert_eq!(
            app.workspace.command_history,
            direct.workspace.command_history
        );
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(
            json["steps"][0]["effect"]["unit_id"],
            expected.unit_id.to_string()
        );
    }
}
