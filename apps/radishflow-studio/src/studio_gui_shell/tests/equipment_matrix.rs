use super::equipment_editor::change;
use super::equipment_export::{apply, csv_records};
use super::equipment_rating_editor::fits;
use super::equipment_rating_export::{fixture, member};
use super::*;
use equipment::CalculationTableFormat as Format;
use rf_model::equipment::*;
use rf_types::units::DisplayUnitSet;
use rf_ui::engineering::{EquipmentCaseKey, EquipmentResultStatus, editor::*};
use serde_json::{Value, json};

fn second() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: member().task_id,
        case_id: "second".into(),
    }
}
fn third() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: member().task_id,
        case_id: "third".into(),
    }
}
fn matrix(name: &str) -> (ReadyAppState, PathBuf) {
    fixture(name, |v| {
        let t = &mut v["engineering"]["tasks"][0];
        t["method"]["version"] = json!(2);
        t["geometry"]["inputs"]["internalDiameter"]["valueSi"] =
            json!((8.0 / std::f64::consts::PI).sqrt());
        t["geometry"]["inputs"]["workingLiquidHeight"]["valueSi"] = json!(1.0);
        t["geometry"]["axialSections"] = json!({});
        t["cases"][0]["name"] = json!("R03 · gas high load");
        t["cases"][0]["inputs"]["gasActualVolumeFlow"]["valueSi"] = json!(1.5);
        t["cases"][0]["inputs"]["liquidActualVolumeFlow"]["valueSi"] = json!(0.005);
        let mut second = t["cases"][0].clone();
        second["id"] = json!("second");
        second["name"] = json!("R04 · liquid high load");
        second["inputs"]["gasActualVolumeFlow"]["valueSi"] = json!(0.8);
        second["inputs"]["liquidActualVolumeFlow"]["valueSi"] = json!(0.015);
        let mut third = t["cases"][0].clone();
        third["id"] = json!("third");
        third["name"] = json!("R05 · gas tie");
        t["cases"].as_array_mut().unwrap().extend([second, third]);
        v["engineering"]["sources"][0]["title"] = json!("=SUM(1,2)\n\"合成来源\"");
    })
}
fn rate(app: &mut ReadyAppState, key: EquipmentCaseKey) {
    app.equipment_command(EquipmentEditorCommand::CalculateRating(key));
}
fn encoded(app: &mut ReadyAppState, format: Format, path: &std::path::Path) {
    let out = app
        .capture_matrix_output(&member().task_id, format)
        .unwrap();
    app.export_equipment_to_path(out, path.into());
}
fn read(path: &std::path::Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}
fn row<'a>(
    table: &'a Value,
    case: Option<&str>,
    section: &str,
    id: &str,
    field: &str,
) -> &'a Value {
    table["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["caseId"].as_str() == case
                && r["section"] == section
                && r["recordId"] == id
                && r["field"] == field
        })
        .unwrap_or_else(|| panic!("missing {case:?} {section}/{id}/{field}"))
}
#[test]
fn matrix_output_full_evidence_non_si_unknown_height_ties_and_csv_json_parity() {
    let (mut app, project) = matrix("matrix-output");
    for key in [member(), second(), third()] {
        rate(&mut app, key);
    }
    let before = app.platform_host.modeling_inputs().workspace.clone();
    let jp = project.with_file_name("matrix.json");
    let cp = project.with_file_name("matrix.csv");
    encoded(&mut app, Format::Json, &jp);
    encoded(&mut app, Format::Csv, &cp);
    assert_eq!(before, app.platform_host.modeling_inputs().workspace);
    let t = read(&jp);
    assert_eq!(t["format"], "radishflow.separator-rating-matrix");
    assert_eq!(t["version"], 1);
    assert_eq!(row(&t, None, "matrix", "", "coverage")["text"], "complete");
    assert_eq!(
        row(&t, None, "matrix", "", "allCasesWithinCriteria")["text"],
        "true"
    );
    assert_eq!(
        row(&t, None, "control", "gas", "caseId.0")["text"],
        member().case_id.as_str()
    );
    assert_eq!(row(&t, None, "control", "gas", "caseId.1")["text"], "third");
    assert_eq!(
        row(&t, None, "control", "liquid", "caseId.0")["text"],
        "second"
    );
    assert!(
        (row(&t, None, "control", "gas", "maximumLoadFraction")["valueSi"]
            .as_f64()
            .unwrap()
            - 0.9375)
            .abs()
            < 1e-14
    );
    assert!(
        (row(&t, None, "control", "liquid", "maximumLoadFraction")["valueSi"]
            .as_f64()
            .unwrap()
            - 0.9)
            .abs()
            < 1e-14
    );
    for key in [member(), second(), third()] {
        let c = key.case_id.as_str();
        assert_eq!(
            row(&t, Some(c), "member", c, "resultStatus")["text"],
            "current"
        );
        assert_eq!(
            row(
                &t,
                Some(c),
                "input",
                "requiredWorkingTime",
                "effectiveValue"
            )["displayUnitId"],
            "minute"
        );
        assert_eq!(
            row(&t, Some(c), "result", c, "tangentToTangentHeight")["status"],
            "unknown"
        );
        assert!(
            t["rows"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["caseId"] == c && r["section"] == "source")
        );
    }
    let records = csv_records(&fs::read_to_string(cp).unwrap());
    let rows = t["rows"].as_array().unwrap();
    assert_eq!(records.len(), rows.len() + 1);
    assert_eq!(records[0].len(), 12);
    for (csv, json) in records[1..].iter().zip(rows) {
        for (i, field) in records[0].iter().enumerate() {
            let value = &json[field];
            if let Some(n) = value.as_f64() {
                assert!(
                    (csv[i].parse::<f64>().unwrap() - n).abs() <= f64::EPSILON * n.abs().max(1.0)
                );
            } else {
                let mut text = value.as_str().unwrap_or_default().to_owned();
                if text.starts_with('=') {
                    text.insert(0, '\'');
                }
                assert_eq!(csv[i], text, "{field}: {json}");
            }
        }
    }
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}
#[test]
fn matrix_output_keeps_empty_failed_stale_rows_without_stale_numerical_results() {
    let (mut app, project) = matrix("matrix-partial");
    let out = project.with_file_name("matrix-empty.json");
    encoded(&mut app, Format::Json, &out);
    let t = read(&out);
    assert_eq!(row(&t, None, "matrix", "", "currentCount")["text"], "0");
    assert!(row(&t, None, "control", "gas", "maximumLoadFraction")["valueSi"].is_null());
    rate(&mut app, member());
    rate(&mut app, second());
    for (key, value) in [(second(), "6000"), (third(), "-1")] {
        let target = EquipmentEditorTarget::RatingCase(key);
        app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
        change(&mut app, &target, |d| {
            let EquipmentDraft::RatingCase(c) = d else {
                panic!()
            };
            c.inputs
                .get_mut(&SeparatorRatingInput::GasVolumeFlow)
                .unwrap()
                .value
                .set_text(value.into());
        });
        apply(&mut app, &target);
    }
    rate(&mut app, third());
    let out = project.with_file_name("matrix-partial.json");
    encoded(&mut app, Format::Json, &out);
    let t = read(&out);
    assert_eq!(
        row(&t, None, "matrix", "", "coverage")["text"],
        "incomplete"
    );
    assert_eq!(
        row(&t, Some("second"), "member", "second", "resultStatus")["text"],
        "stale"
    );
    assert_eq!(
        row(&t, Some("third"), "member", "third", "resultStatus")["text"],
        "failed"
    );
    assert!(
        t["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["caseId"] == "third" && r["section"] == "failure")
    );
    assert!(
        !t["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["caseId"] == "second" && r["section"] == "result")
    );
    rate(&mut app, second());
    let out = project.with_file_name("matrix-exceeded.json");
    encoded(&mut app, Format::Json, &out);
    let t = read(&out);
    assert_eq!(
        row(&t, None, "matrix", "", "coverage")["text"],
        "incomplete"
    );
    assert_eq!(
        row(&t, None, "matrix", "", "subsetConclusion")["text"],
        "exceeded"
    );
    assert!(
        row(&t, Some("second"), "result", "second", "gasMarginFraction")["valueSi"]
            .as_f64()
            .unwrap()
            < 0.0
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

#[test]
fn matrix_overwrite_rechecks_missing_execution_membership_drafts_units_and_reopen() {
    for mutation in [
        "missing-to-current",
        "rerun",
        "failed-rerun",
        "add",
        "delete-undo",
        "draft",
        "units",
        "source",
        "reopen",
    ] {
        let (mut app, project) = matrix("matrix-output-recheck");
        rate(&mut app, member());
        if mutation == "failed-rerun" {
            let target = EquipmentEditorTarget::RatingCase(second());
            app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
            change(&mut app, &target, |d| {
                let EquipmentDraft::RatingCase(c) = d else {
                    panic!()
                };
                c.inputs
                    .get_mut(&SeparatorRatingInput::GasVolumeFlow)
                    .unwrap()
                    .value
                    .set_text("-1".into());
            });
            apply(&mut app, &target);
            rate(&mut app, second());
        }
        let out = project.with_file_name("matrix.json");
        fs::write(&out, "original").unwrap();
        encoded(&mut app, Format::Json, &out);
        assert!(app.equipment.pending_export.is_some());
        match mutation {
            "missing-to-current" => rate(&mut app, second()),
            "rerun" => rate(&mut app, member()),
            "failed-rerun" => rate(&mut app, second()),
            "add" => app.equipment_command(EquipmentEditorCommand::AddRatingCase {
                task: member().task_id,
                copy_from: None,
            }),
            "delete-undo" => {
                app.equipment_command(EquipmentEditorCommand::Delete(
                    EquipmentEditorTarget::RatingCase(third()),
                ));
                let key = app
                    .platform_host
                    .modeling_inputs()
                    .workspace
                    .equipment_editor
                    .confirmation()
                    .unwrap()
                    .key();
                app.equipment_command(EquipmentEditorCommand::Confirm(key));
                app.equipment_command(EquipmentEditorCommand::History { redo: false });
            }
            "draft" => {
                let target = EquipmentEditorTarget::RatingCase(second());
                app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
                change(&mut app, &target, |d| {
                    let EquipmentDraft::RatingCase(c) = d else {
                        panic!()
                    };
                    c.name = "unsubmitted".into();
                });
            }
            "units" => {
                app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(
                    DisplayUnitSet::si(),
                ))
                .unwrap();
            }
            "source" => {
                let target = EquipmentEditorTarget::Source("source-test".into());
                app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
                change(&mut app, &target, |d| {
                    let EquipmentDraft::Source(s) = d else {
                        panic!()
                    };
                    s.publication_version = "changed".into();
                });
                apply(&mut app, &target);
            }
            "reopen" => {
                app.open_project(project.clone(), "reopen matrix");
                rate(&mut app, member());
            }
            _ => unreachable!(),
        }
        app.confirm_equipment_export();
        assert_eq!(fs::read_to_string(&out).unwrap(), "original", "{mutation}");
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }
}
#[test]
fn matrix_metadata_and_unrelated_drafts_keep_capture_while_row_export_ignores_other_rows() {
    let (mut app, project) = matrix("matrix-scoped-output");
    rate(&mut app, member());
    let path = project.with_file_name("matrix.json");
    fs::write(&path, "old").unwrap();
    encoded(&mut app, Format::Json, &path);
    app.equipment_command(EquipmentEditorCommand::ReorderRatingCases {
        task: member().task_id,
        cases: vec![third().case_id, second().case_id, member().case_id],
    });
    let target = EquipmentEditorTarget::RatingCase(second());
    app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
    change(&mut app, &target, |d| {
        let EquipmentDraft::RatingCase(c) = d else {
            panic!()
        };
        c.name = "new label".into();
    });
    apply(&mut app, &target);
    app.equipment_command(EquipmentEditorCommand::CreateSource);
    let source = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .active()
        .unwrap()
        .clone();
    change(&mut app, &source, |d| {
        let EquipmentDraft::Source(s) = d else {
            panic!()
        };
        s.title = "unrelated draft".into();
    });
    app.confirm_equipment_export();
    let t = read(&path);
    assert_eq!(
        row(&t, Some("second"), "member", "second", "name")["text"],
        "R04 · liquid high load"
    );
    assert_eq!(
        row(&t, Some("third"), "member", "third", "order")["text"],
        "2"
    );
    assert!(
        !fs::read_to_string(&path)
            .unwrap()
            .contains("unrelated draft")
    );
    app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
    change(&mut app, &target, |d| {
        let EquipmentDraft::RatingCase(c) = d else {
            panic!()
        };
        c.inputs
            .get_mut(&SeparatorRatingInput::GasVolumeFlow)
            .unwrap()
            .value
            .set_text("1e-".into());
    });
    assert!(
        app.capture_matrix_output(&member().task_id, Format::Json)
            .is_none()
    );
    let output = app.capture_rating_output(&member(), Format::Json).unwrap();
    let row_path = project.with_file_name("row.json");
    app.export_equipment_to_path(output, row_path.clone());
    assert_eq!(
        read(&row_path)["format"],
        "radishflow.separator-rating-table"
    );
    assert!(
        app.capture_equipment_output(&member().task_id, Format::Json)
            .is_none()
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}
#[test]
fn matrix_selection_f5_reorder_delete_and_undo_follow_case_identity() {
    let (mut app, project) = matrix("matrix-selection");
    app.equipment_command(EquipmentEditorCommand::Select(
        EquipmentEditorTarget::RatingCase(second()),
    ));
    app.calculate_active_equipment();
    let s = &app.platform_host.modeling_inputs().workspace.engineering;
    assert_eq!(
        s.case_result_status(&second()),
        EquipmentResultStatus::Current
    );
    assert_eq!(
        s.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    app.equipment_command(EquipmentEditorCommand::ReorderRatingCases {
        task: member().task_id,
        cases: vec![third().case_id, member().case_id, second().case_id],
    });
    app.equipment_command(EquipmentEditorCommand::Select(
        EquipmentEditorTarget::RatingGeometry(member().task_id),
    ));
    app.calculate_active_equipment();
    assert_eq!(
        app.equipment.rating_selection[&member().task_id],
        second().case_id
    );
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    app.equipment_command(EquipmentEditorCommand::Select(
        EquipmentEditorTarget::RatingCase(second()),
    ));
    app.equipment_command(EquipmentEditorCommand::Delete(
        EquipmentEditorTarget::RatingCase(second()),
    ));
    let key = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .confirmation()
        .unwrap()
        .key();
    app.equipment_command(EquipmentEditorCommand::Confirm(key));
    assert_ne!(
        app.equipment.rating_selection[&member().task_id],
        second().case_id
    );
    app.equipment_command(EquipmentEditorCommand::History { redo: false });
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&second()),
        EquipmentResultStatus::Stale
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}
#[test]
fn matrix_and_detail_layouts_fit_both_languages_narrow_and_doubled_text() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for width in [400.0, 1440.0] {
            for scale in [1.0, 2.0] {
                let (mut app, project) = matrix("matrix-layout");
                rate(&mut app, member());
                rate(&mut app, second());
                app.locale = locale;
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                fonts::configure_studio_fonts(&ctx);
                ctx.style_mut(|s| {
                    for font in s.text_styles.values_mut() {
                        font.size *= scale;
                    }
                });
                for overview in [true, false] {
                    app.equipment_command(EquipmentEditorCommand::Select(
                        EquipmentEditorTarget::RatingCase(second()),
                    ));
                    app.equipment
                        .rating_overview
                        .insert(member().task_id, overview);
                    for section in [
                        equipment::EquipmentSection::Inputs,
                        equipment::EquipmentSection::Results,
                        equipment::EquipmentSection::Evidence,
                    ] {
                        app.equipment.section = section;
                        for _ in 0..2 {
                            let output = ctx.run(
                                egui::RawInput {
                                    screen_rect: Some(egui::Rect::from_min_size(
                                        egui::Pos2::ZERO,
                                        egui::vec2(width, 1100.0),
                                    )),
                                    ..Default::default()
                                },
                                |ctx| app.render_equipment_page(ctx),
                            );
                            let values = output
                                .platform_output
                                .accesskit_update
                                .unwrap()
                                .nodes
                                .iter()
                                .filter(|(_, n)| {
                                    matches!(
                                        n.role(),
                                        egui::accesskit::Role::TextInput
                                            | egui::accesskit::Role::MultilineTextInput
                                    )
                                })
                                .filter_map(|(_, n)| n.value().map(str::to_owned))
                                .collect::<Vec<_>>();
                            for shape in output.shapes {
                                fits(
                                    &shape.shape,
                                    shape.clip_rect,
                                    &values,
                                    &format!(
                                        "matrix {locale:?} {width} {scale} {overview} {section:?}"
                                    ),
                                );
                            }
                        }
                    }
                }
                fs::remove_dir_all(project.parent().unwrap()).unwrap();
            }
        }
    }
}

#[test]
fn matrix_case_switch_restores_numeric_focus_and_invalid_draft_without_blocking_other_rows() {
    use egui::accesskit::{Action, ActionRequest, Role};
    let (mut app, project) = matrix("matrix-focus");
    app.locale = StudioShellLocale::En;
    app.equipment_command(EquipmentEditorCommand::Select(
        EquipmentEditorTarget::RatingCase(second()),
    ));
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let frame = |app: &mut ReadyAppState, events| {
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 1000.0),
            )),
            focused: true,
            events,
            ..Default::default()
        });
        app.dispatch_shortcuts(&ctx);
        app.render_equipment_page(&ctx);
        ctx.end_pass().platform_output.accesskit_update.unwrap()
    };
    let key = |key, modifiers| egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    };
    frame(&mut app, vec![]);
    let tree = frame(&mut app, vec![]);
    let input = tree
        .nodes
        .iter()
        .find(|(_, n)| {
            n.role() == Role::TextInput && n.value().is_some_and(|v| v.starts_with("2880"))
        })
        .map(|(id, _)| *id)
        .expect("selected second gas flow");
    frame(
        &mut app,
        vec![egui::Event::AccessKitActionRequest(ActionRequest {
            target: input,
            action: Action::Focus,
            data: None,
        })],
    );
    frame(&mut app, vec![key(egui::Key::A, egui::Modifiers::COMMAND)]);
    frame(&mut app, vec![egui::Event::Text("1e-".into())]);
    frame(&mut app, vec![]);
    app.equipment_command(EquipmentEditorCommand::Select(
        EquipmentEditorTarget::RatingCase(third()),
    ));
    frame(&mut app, vec![]);
    frame(&mut app, vec![]);
    frame(&mut app, vec![key(egui::Key::F5, egui::Modifiers::NONE)]);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&third()),
        EquipmentResultStatus::Current
    );
    app.equipment_command(EquipmentEditorCommand::Select(
        EquipmentEditorTarget::RatingCase(second()),
    ));
    frame(&mut app, vec![]);
    let tree = frame(&mut app, vec![]);
    assert_eq!(tree.focus, input);
    let w = &app.platform_host.modeling_inputs().workspace;
    let target = EquipmentEditorTarget::RatingCase(second());
    let EquipmentDraft::RatingCase(c) = w.equipment_editor.draft(&target).unwrap().value() else {
        panic!()
    };
    assert_eq!(
        c.inputs[&SeparatorRatingInput::GasVolumeFlow].value.text(),
        "1e-"
    );
    frame(&mut app, vec![key(egui::Key::F5, egui::Modifiers::NONE)]);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&second()),
        EquipmentResultStatus::Missing
    );
    assert!(app.capture_rating_output(&third(), Format::Json).is_some());
    assert!(
        app.capture_matrix_output(&member().task_id, Format::Json)
            .is_none()
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}
#[test]
fn matrix_upgrade_and_delete_confirmation_are_localized_readable_and_cancel_preserves_inputs() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        let (mut app, project) = fixture("matrix-upgrade-modal", |_| {});
        app.locale = locale;
        app.equipment_command(EquipmentEditorCommand::AddRatingCase {
            task: member().task_id,
            copy_from: Some(member().case_id),
        });
        let before = app
            .platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .clone();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        fonts::configure_studio_fonts(&ctx);
        ctx.style_mut(|s| {
            for f in s.text_styles.values_mut() {
                f.size *= 2.0;
            }
        });
        for _ in 0..2 {
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 1000.0),
                    )),
                    ..Default::default()
                },
                |ctx| app.render_equipment_confirmation(ctx),
            );
            for shape in out.shapes {
                fits(&shape.shape, shape.clip_rect, &[], "matrix upgrade modal");
            }
        }
        app.equipment_command(EquipmentEditorCommand::CancelConfirmation);
        assert_eq!(
            app.platform_host.modeling_inputs().workspace.engineering,
            before
        );
        app.equipment_command(EquipmentEditorCommand::AddRatingCase {
            task: member().task_id,
            copy_from: None,
        });
        let key = app
            .platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .confirmation()
            .unwrap()
            .key();
        app.equipment_command(EquipmentEditorCommand::Confirm(key));
        let target = app
            .platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .active()
            .unwrap()
            .clone();
        app.equipment_command(EquipmentEditorCommand::Delete(target));
        let before = app
            .platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .clone();
        for _ in 0..2 {
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 1000.0),
                    )),
                    ..Default::default()
                },
                |ctx| app.render_equipment_confirmation(ctx),
            );
            for shape in out.shapes {
                fits(&shape.shape, shape.clip_rect, &[], "matrix delete modal");
            }
        }
        app.equipment_command(EquipmentEditorCommand::CancelConfirmation);
        assert_eq!(
            app.platform_host.modeling_inputs().workspace.engineering,
            before
        );
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }
}
