//! Explicit authoring accepts cycles; execution readiness is a separate operation.
use super::*;
use rf_ui::{CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding};
pub(super) fn connect(
    app: &mut AppState,
    target: &MaterialConnectionTarget,
    changed_at: DateTimeUtc,
) -> rf_types::RfResult<DocumentCommand> {
    if target.stream_id.as_str().trim().is_empty() {
        return Err(RfError::invalid_input("stream identity cannot be blank"));
    }
    let flow = &app.workspace.document.flowsheet;
    let stream = if flow.streams.contains_key(&target.stream_id) {
        CanvasSuggestedStreamBinding::Existing {
            stream_id: target.stream_id.clone(),
        }
    } else {
        if app.workspace.stream_identity_reserved(&target.stream_id) {
            return Err(RfError::invalid_input(
                "stream identity was previously allocated or referenced by a plan",
            ));
        }
        CanvasSuggestedStreamBinding::Create {
            stream: rf_model::MaterialStreamState::new(
                target.stream_id.clone(),
                target.stream_id.as_str(),
            ),
        }
    };
    let connection = CanvasSuggestedMaterialConnection {
        source_unit_id: target.source.unit_id.clone(),
        source_port: target.source.port.clone(),
        sink_unit_id: target.sink.as_ref().map(|s| s.unit_id.clone()),
        sink_port: target.sink.as_ref().map(|s| s.port.clone()),
        stream,
    };
    app.validate_explicit_material_connection(&connection)?;
    app.commit_material_connection(&connection, changed_at)
        .map(|(command, _)| command)
}
