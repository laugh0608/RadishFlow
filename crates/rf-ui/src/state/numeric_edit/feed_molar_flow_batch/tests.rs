use super::*;
use crate::variable_browser::VariableWatchCommand;
use std::time::UNIX_EPOCH;

fn setup() -> (AppState, Vec<VariableId>) {
    let mut flowsheet = Flowsheet::new("Dual feed");
    for (id, flow) in [("a", 2.0), ("b", 3.0), ("c", 4.0)] {
        let mut stream = MaterialStreamState::new(id, id);
        stream.total_molar_flow_mol_s = flow;
        flowsheet.insert_stream(stream).unwrap();
        flowsheet
            .insert_unit(rf_unitops::build_feed_node(format!("feed-{id}"), id, id))
            .unwrap();
    }
    let mut app = AppState::new(crate::FlowsheetDocument::new(
        flowsheet,
        crate::DocumentMetadata::new("batch", "Dual feed", UNIX_EPOCH),
    ));
    let workspace = &mut app.workspace;
    workspace.install_saved_object_identities(workspace.prepare_object_identities().unwrap());
    let ids: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|id| VariableId {
            document: "batch".into(),
            object: ObjectId::Stream(id.into()),
            section: VariableSection::Inputs,
            field: VariableField::MolarFlow,
        })
        .collect();
    for id in &ids {
        workspace
            .edit_variable_watch_list(&VariableWatchCommand::Add(id.clone()))
            .unwrap();
    }
    (app, ids)
}

fn draft(app: &mut AppState, id: &VariableId, raw: &str) {
    let generation = match app.workspace.numeric_edit(id) {
        Ok(session) => session.generation(),
        Err(_) => app.begin_numeric_edit(id.clone(), None).unwrap(),
    };
    app.edit_numeric(id, generation, NumericEditEvent::ReplaceText(raw.into()))
        .unwrap();
}

fn flows(app: &AppState) -> [f64; 2] {
    ["a", "b"].map(|id| {
        app.workspace.document.flowsheet.streams[&StreamId::new(id)].total_molar_flow_mol_s
    })
}

#[test]
fn invalid_draft_rejects_every_item_and_valid_batch_is_one_edit_and_undo() {
    let (mut app, ids) = setup();
    let batch = &ids[..2];
    draft(&mut app, &batch[0], "36 kmol/h");
    for raw in ["1e-", "-5"] {
        draft(&mut app, &batch[1], raw);
        let rejected = app
            .workspace
            .check_feed_molar_flow_batch(batch)
            .unwrap_err();
        let FeedMolarFlowBatchError::InvalidDrafts(items) = rejected else {
            panic!("{rejected:?}")
        };
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].0, batch[1]);
        assert!(match raw {
            "1e-" => matches!(items[0].1, NumericEditError::Incomplete),
            _ => matches!(items[0].1, NumericEditError::Rejected(_)),
        });
        assert_eq!(flows(&app), [2.0, 3.0]);
        assert_eq!(app.workspace.document.revision, 0);
        assert!(app.workspace.command_history.is_empty());
        assert_eq!(
            app.workspace.numeric_edit(&batch[0]).unwrap().raw_text(),
            "36 kmol/h"
        );
        assert_eq!(
            app.workspace.numeric_edit(&batch[1]).unwrap().raw_text(),
            raw
        );
    }
    draft(&mut app, &batch[1], "5");
    draft(&mut app, &ids[2], "7");
    let unrelated = app.workspace.numeric_edit(&ids[2]).unwrap().generation();
    let checked = app.workspace.check_feed_molar_flow_batch(batch).unwrap();
    let mut old_run = app.prepare_flow_run().into_receipt();
    let receipt = app
        .apply_feed_molar_flow_batch(&checked, UNIX_EPOCH)
        .unwrap();
    assert_eq!(
        receipt,
        FeedMolarFlowBatchReceipt {
            revision: 1,
            changed_count: 2
        }
    );
    assert_eq!(flows(&app), [10.0, 5.0]);
    assert_eq!(app.workspace.command_history.len(), 1);
    assert!(
        batch
            .iter()
            .all(|id| app.workspace.numeric_edit(id).is_err())
    );
    assert_eq!(app.workspace.numeric_edit(&ids[2]).unwrap().raw_text(), "7");
    assert_eq!(
        app.accept_flow_run(&mut old_run),
        crate::FlowRunDisposition::Discarded(crate::FlowRunDiscardReason::InputChanged)
    );
    app.cancel_numeric_edit(&ids[2], unrelated).unwrap();
    app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
    assert_eq!(flows(&app), [2.0, 3.0]);
    app.redo_document_command(UNIX_EPOCH).unwrap().unwrap();
    assert_eq!(flows(&app), [10.0, 5.0]);
}

#[test]
fn rejects_old_checks_and_scope_changes_and_counts_only_changed_values() {
    let (mut app, ids) = setup();
    let batch = &ids[..2];
    draft(&mut app, &batch[0], "2");
    draft(&mut app, &batch[1], "3");
    let checked = app.workspace.check_feed_molar_flow_batch(batch).unwrap();
    draft(&mut app, &batch[1], "4");
    assert_eq!(
        app.apply_feed_molar_flow_batch(&checked, UNIX_EPOCH),
        Err(FeedMolarFlowBatchError::StaleCheck)
    );
    let checked = app.workspace.check_feed_molar_flow_batch(batch).unwrap();
    app.workspace.reset_flow_run_session();
    assert_eq!(
        app.apply_feed_molar_flow_batch(&checked, UNIX_EPOCH),
        Err(FeedMolarFlowBatchError::StaleCheck)
    );
    assert!(
        app.workspace
            .check_feed_molar_flow_batch(&[batch[0].clone(), batch[0].clone()])
            .is_err()
    );
    let mut result = batch[0].clone();
    result.section = VariableSection::Results;
    assert!(!app.workspace.supports_feed_molar_flow_batch_item(&result));
    let mut foreign = batch[0].clone();
    foreign.document = "another".into();
    assert!(!app.workspace.supports_feed_molar_flow_batch_item(&foreign));
    let checked = app.workspace.check_feed_molar_flow_batch(batch).unwrap();
    let reference = app
        .workspace
        .watch_reference(batch[1].clone())
        .unwrap()
        .to_string();
    app.workspace
        .edit_variable_watch_list(&VariableWatchCommand::Remove(reference))
        .unwrap();
    assert_eq!(
        app.apply_feed_molar_flow_batch(&checked, UNIX_EPOCH),
        Err(FeedMolarFlowBatchError::StaleCheck)
    );
    app.workspace
        .edit_variable_watch_list(&VariableWatchCommand::Add(batch[1].clone()))
        .unwrap();
    let checked = app.workspace.check_feed_molar_flow_batch(batch).unwrap();
    assert_eq!(
        app.apply_feed_molar_flow_batch(&checked, UNIX_EPOCH)
            .unwrap()
            .changed_count,
        1
    );
    draft(&mut app, &batch[0], "2");
    draft(&mut app, &batch[1], "4");
    let checked = app.workspace.check_feed_molar_flow_batch(batch).unwrap();
    assert_eq!(
        app.apply_feed_molar_flow_batch(&checked, UNIX_EPOCH)
            .unwrap(),
        FeedMolarFlowBatchReceipt {
            revision: 1,
            changed_count: 0
        }
    );
    assert_eq!(app.workspace.command_history.len(), 1);

    draft(&mut app, &batch[0], "8");
    draft(&mut app, &batch[1], "9");
    let checked = app.workspace.check_feed_molar_flow_batch(batch).unwrap();
    app.disconnect_material_port(&"feed-b".into(), rf_unitops::FEED_OUTLET_PORT, UNIX_EPOCH)
        .unwrap()
        .unwrap();
    assert_eq!(
        app.apply_feed_molar_flow_batch(&checked, UNIX_EPOCH),
        Err(FeedMolarFlowBatchError::StaleCheck)
    );
    assert!(matches!(
        app.workspace.check_feed_molar_flow_batch(batch),
        Err(FeedMolarFlowBatchError::InvalidDrafts(items))
            if items == vec![(batch[1].clone(), NumericEditError::UnsupportedField)]
    ));
    assert_eq!(flows(&app), [2.0, 4.0]);
    assert_eq!(
        app.workspace.numeric_edit(&batch[0]).unwrap().raw_text(),
        "8"
    );
    assert_eq!(
        app.workspace.numeric_edit(&batch[1]).unwrap().raw_text(),
        "9"
    );
}
