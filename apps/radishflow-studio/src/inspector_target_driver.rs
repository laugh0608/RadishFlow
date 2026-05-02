use rf_ui::{AppState, InspectorTarget};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorTargetFocusOutcome {
    pub requested_target: InspectorTarget,
    pub applied_target: Option<InspectorTarget>,
    pub active_target: Option<InspectorTarget>,
}

pub fn focus_inspector_target(
    app_state: &mut AppState,
    target: InspectorTarget,
) -> InspectorTargetFocusOutcome {
    let applied_target = app_state.focus_inspector_target(target.clone());
    let active_target = app_state.workspace.drafts.active_target.clone();

    InspectorTargetFocusOutcome {
        requested_target: target,
        applied_target,
        active_target,
    }
}

#[cfg(test)]
mod tests {
    use rf_model::{Flowsheet, MaterialStreamState, UnitNode, UnitPort};
    use rf_types::{PortDirection, PortKind, StreamId, UnitId};
    use rf_ui::{AppState, DocumentMetadata, FlowsheetDocument, InspectorTarget};

    use crate::focus_inspector_target;

    #[test]
    fn inspector_target_driver_focuses_existing_stream() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_unit(UnitNode::new(
                "feed-1",
                "Feed",
                "feed",
                vec![UnitPort::new(
                    "outlet",
                    PortDirection::Outlet,
                    PortKind::Material,
                    Some("stream-feed".into()),
                )],
            ))
            .expect("expected unit insert");
        flowsheet
            .insert_stream(MaterialStreamState::new("stream-feed", "Feed stream"))
            .expect("expected stream insert");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Demo", std::time::UNIX_EPOCH),
        ));

        let outcome = focus_inspector_target(
            &mut app_state,
            InspectorTarget::Stream(StreamId::new("stream-feed")),
        );

        assert_eq!(
            outcome.applied_target,
            Some(InspectorTarget::Stream(StreamId::new("stream-feed")))
        );
        assert_eq!(
            outcome.active_target,
            Some(InspectorTarget::Stream(StreamId::new("stream-feed")))
        );
    }

    #[test]
    fn inspector_target_driver_reports_missing_unit_without_changing_focus() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_unit(UnitNode::new("feed-1", "Feed", "feed", Vec::new()))
            .expect("expected unit insert");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Demo", std::time::UNIX_EPOCH),
        ));
        focus_inspector_target(&mut app_state, InspectorTarget::Unit(UnitId::new("feed-1")));

        let outcome = focus_inspector_target(
            &mut app_state,
            InspectorTarget::Unit(UnitId::new("missing-unit")),
        );

        assert_eq!(outcome.applied_target, None);
        assert_eq!(
            outcome.active_target,
            Some(InspectorTarget::Unit(UnitId::new("feed-1")))
        );
    }
}
