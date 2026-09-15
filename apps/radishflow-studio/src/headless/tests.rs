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
