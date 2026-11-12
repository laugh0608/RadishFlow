use super::*;
use rf_types::{StreamId, UnitId};
use rf_ui::project_query::{ProjectObjectId, ProjectQuery};
use rf_ui::variable_browser::{VariableField, VariableSection};
use std::time::{Instant, UNIX_EPOCH};

fn app() -> ReadyAppState {
    super::super::tests::ready_app_state(&super::super::tests::synced_workspace_config())
}
fn flow_selection(
    app: &ReadyAppState,
    object: ObjectId,
    section: VariableSection,
    field: VariableField,
) -> BrowserSelection {
    BrowserSelection {
        object: ProjectQuery::new(&app.platform_host.modeling_inputs().workspace)
            .reference(ProjectObjectId::Flow(object)),
        field: Some(ProjectField::Flow { section, field }),
    }
}
fn synchronize(app: &mut ReadyAppState) {
    app.variable_browser
        .synchronize(&app.platform_host.modeling_inputs().workspace, app.locale);
}
fn frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 860.),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            app.render_variable_browser(ctx);
        },
    )
}

#[test]
fn browser_stable_reference_enables_saves_reopens_and_locates_without_mutation() {
    let mut app = app();
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("run_panel.run_manual");
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_result()
            .status(),
        rf_ui::result_query::ResultStatus::Current
    );
    app.open_variable_browser();
    synchronize(&mut app);
    let target = std::env::temp_dir().join(format!(
        "radishflow-reference-browser-{}-{}.rfproj.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let input = flow_selection(
        &app,
        ObjectId::Unit("heater-1".into()),
        VariableSection::Inputs,
        VariableField::OutletTemperature,
    );
    let result = flow_selection(
        &app,
        ObjectId::Stream("stream-heated".into()),
        VariableSection::Results,
        VariableField::Temperature,
    );
    assert!(app.copy_stable_reference(&input).is_err());
    app.request_object_identity_initialization();
    app.prepare_identity_destination(target.clone());
    assert!(!target.exists());
    app.confirm_identity_initialization();
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_result()
            .status(),
        rf_ui::result_query::ResultStatus::Current
    );
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .document_path
            .as_ref(),
        Some(&target)
    );
    let input_ref = app.copy_stable_reference(&input).unwrap();
    let result_ref = app.copy_stable_reference(&result).unwrap();
    assert!(input_ref.ends_with("/inputs/outlet_temperature"));
    assert!(result_ref.ends_with("/results/temperature"));
    let original_bytes = std::fs::read(&target).unwrap();

    app.open_project(target.clone(), "project");
    app.reconcile_browser_session();
    app.open_variable_browser();
    synchronize(&mut app);
    app.open_stable_reference(&result_ref);
    assert_eq!(app.variable_browser.selected, Some(result.clone()));
    assert!(app.browser_expanded());
    assert!(app.browser_copy_value(&result).is_err());
    assert_eq!(app.copy_stable_reference(&result).unwrap(), result_ref);

    app.dispatch_ui_command("inspector.focus_unit:heater-1");
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id("unit:heater-1:outlet_temperature_k"),
        "1e-",
    );
    let before = app.platform_host.modeling_inputs().workspace.clone();
    app.open_stable_reference(&input_ref);
    assert_eq!(app.variable_browser.selected, Some(result));
    let navigation = app.variable_browser.pending_navigation.take().unwrap();
    assert!(app.navigate_browser(navigation));
    assert_eq!(app.variable_browser.selected, Some(input));
    assert_eq!(
        app.variable_browser.pending_focus.as_ref().unwrap().field,
        VariableField::OutletTemperature
    );
    let after = &app.platform_host.modeling_inputs().workspace;
    assert_eq!(after.drafts, before.drafts);
    assert_eq!(after.document, before.document);
    assert_eq!(std::fs::read(&target).unwrap(), original_bytes);
    std::fs::remove_file(&target).unwrap();
    let _ = std::fs::remove_file(format!("{}.rfstudio-layout.json", target.display()));
}

#[test]
fn browser_copy_rechecks_current_stale_missing_and_session_without_mutation() {
    let mut app = app();
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("run_panel.run_manual");
    app.open_variable_browser();
    synchronize(&mut app);
    let selected = flow_selection(
        &app,
        ObjectId::Stream(StreamId::new("stream-heated")),
        VariableSection::Results,
        VariableField::Temperature,
    );
    let input = flow_selection(
        &app,
        ObjectId::Unit(UnitId::new("heater-1")),
        VariableSection::Inputs,
        VariableField::OutletTemperature,
    );
    let before = app.platform_host.modeling_inputs().workspace.clone();
    assert_eq!(
        before.flow_result().status(),
        rf_ui::result_query::ResultStatus::Current
    );
    assert!(app.browser_copy_value(&selected).unwrap().contains('K'));
    app.variable_browser.selected = Some(selected.clone());
    app.variable_browser.query = "temperature".into();
    synchronize(&mut app);
    let rebuilds = app.variable_browser.cache.rebuilds;
    for _ in 0..10 {
        synchronize(&mut app);
    }
    assert_eq!(app.variable_browser.cache.rebuilds, rebuilds);
    assert_eq!(app.platform_host.modeling_inputs().workspace, before);
    super::super::tests::commit_unit_parameter(
        &mut app,
        "heater-1",
        "unit:heater-1:outlet_temperature_k",
        "340",
    );
    assert!(app.browser_copy_value(&selected).is_err());
    let stale = read_selection(
        &app.platform_host.modeling_inputs().workspace,
        &selected,
        app.locale,
    )
    .unwrap();
    assert_eq!(stale.status, "Stale");
    assert!(stale.canonical.is_none());
    assert!(stale.value.starts_with('—'));
    assert!(app.browser_copy_value(&input).unwrap().contains("340"));
    app.dispatch_ui_command("run_panel.run_manual");
    synchronize(&mut app);
    assert_eq!(app.variable_browser.selected, Some(selected.clone()));
    assert_eq!(app.variable_browser.query, "temperature");
    assert!(app.browser_copy_value(&selected).unwrap().contains("340"));
    let path = app
        .platform_host
        .modeling_inputs()
        .workspace
        .document_path
        .clone()
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    app.open_project(path.clone(), "project");
    assert!(app.browser_copy_value(&selected).is_err());
    app.reconcile_browser_session();
    assert!(app.variable_browser.selected.is_none());
    assert!(!app.variable_browser.open);
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn browser_navigation_preserves_drafts_and_uses_typed_field_focus() {
    let mut app = app();
    app.open_variable_browser();
    synchronize(&mut app);
    let selected = flow_selection(
        &app,
        ObjectId::Unit(UnitId::new("heater-1")),
        VariableSection::Inputs,
        VariableField::OutletTemperature,
    );
    app.variable_browser.selected = Some(selected.clone());
    app.dispatch_ui_command("inspector.focus_unit:heater-1");
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id("unit:heater-1:outlet_temperature_k"),
        "1e-",
    );
    let before = app.platform_host.modeling_inputs().workspace.clone();
    app.request_browser_navigation(BrowserNavigation::Selection(selected.clone()));
    assert!(app.browser_navigation_pending());
    assert_eq!(app.platform_host.modeling_inputs().workspace, before);
    // Cancel does not change either the editor or the browser identity.
    app.variable_browser.pending_navigation = None;
    assert_eq!(app.variable_browser.selected, Some(selected.clone()));
    assert!(app.navigate_browser(BrowserNavigation::Selection(selected.clone())));
    assert_eq!(
        app.platform_host.modeling_inputs().workspace.drafts,
        before.drafts
    );
    assert_eq!(app.platform_host.document(), &before.document);
    assert!(!app.variable_browser.open);
    assert_eq!(
        app.right_sidebar_tab,
        StudioShellRightSidebarTab::ModuleSettings
    );
    let variable = app.variable_browser.pending_focus.clone().unwrap();
    assert_eq!(variable.field, VariableField::OutletTemperature);
    app.select_browser_variable(variable);
    assert_eq!(app.variable_browser.selected, Some(selected));
    assert!(app.variable_browser.open);
}

fn engineering_app() -> rf_ui::AppState {
    let flow = rf_model::Flowsheet::new("browser-equipment");
    let mut app = rf_ui::AppState::new(rf_ui::FlowsheetDocument::new(
        flow,
        rf_ui::DocumentMetadata::new("browser-equipment", "Equipment", UNIX_EPOCH),
    ));
    let rf_store::project_v3::ProjectV3Read::Editable(project) = rf_store::project_v3::parse(
        include_str!("../../../../../crates/rf-store/tests/fixtures/project-v3-rating.json"),
    )
    .unwrap() else {
        panic!("editable fixture")
    };
    let engineering = project.engineering.unwrap();
    app.workspace.engineering = rf_ui::engineering::EngineeringSession::from_inputs(
        "browser-equipment".into(),
        rf_ui::engineering::EngineeringInputs {
            basis: engineering.basis,
            tasks: engineering.tasks,
        },
    )
    .unwrap();
    app
}

#[test]
fn browser_equipment_directory_resolves_basis_geometry_and_result_entries() {
    let app = engineering_app();
    let before = app.clone();
    let mut state = VariableBrowserState::default();
    state.synchronize(&app.workspace, StudioShellLocale::En);
    let query = ProjectQuery::new(&app.workspace);
    let case = query.reference(ProjectObjectId::Case(
        rf_ui::engineering::EquipmentCaseKey {
            task_id: "rating-1".into(),
            case_id: "rating-case-1".into(),
        },
    ));
    let selected = BrowserSelection {
        object: case.clone(),
        field: Some(ProjectField::RatingInput(
            rf_model::equipment::SeparatorRatingInput::RequiredWorkingTime,
        )),
    };
    let read = read_selection(&app.workspace, &selected, StudioShellLocale::En).unwrap();
    assert!(read.source.contains("Basis"));
    assert!(read.canonical.as_deref().unwrap().contains("120"));
    state.cache.filter("", SectionFilter::Results, None);
    assert!(
        state
            .cache
            .visible
            .iter()
            .any(|i| state.cache.entries[*i].target.object == case)
    );
    assert_eq!(
        state
            .cache
            .objects
            .iter()
            .filter(|o| matches!(o.reference.object, ProjectObjectId::Geometry(_)))
            .count(),
        1
    );
    assert!(
        state
            .cache
            .objects
            .iter()
            .any(|o| matches!(o.reference.object, ProjectObjectId::Source(_)))
    );
    let mut wrong = selected.clone();
    wrong.object.object = ProjectObjectId::Case(rf_ui::engineering::EquipmentCaseKey {
        task_id: "another-task".into(),
        case_id: "rating-case-1".into(),
    });
    assert!(read_selection(&app.workspace, &wrong, StudioShellLocale::En).is_err());
    assert_eq!(app, before);
}

#[test]
fn browser_narrow_double_font_and_escape_are_read_only_in_both_locales() {
    for locale in [StudioShellLocale::En, StudioShellLocale::ZhCn] {
        let mut app = app();
        app.locale = locale;
        app.open_variable_browser();
        synchronize(&mut app);
        app.variable_browser.selected = Some(flow_selection(
            &app,
            ObjectId::Stream(StreamId::new("stream-heated")),
            VariableSection::Inputs,
            VariableField::MolarFlow,
        ));
        app.variable_browser.detail_page = true;
        let before = app.platform_host.modeling_inputs().workspace.clone();
        let ctx = egui::Context::default();
        ctx.style_mut(|s| {
            for font in s.text_styles.values_mut() {
                font.size *= 2.;
            }
        });
        frame(&mut app, &ctx, 400., vec![]);
        frame(
            &mut app,
            &ctx,
            400.,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(!app.variable_browser.detail_page);
        assert!(app.variable_browser.open);
        assert_eq!(app.platform_host.modeling_inputs().workspace, before);
    }
}

#[test]
fn browser_large_index_reuses_metadata_on_warm_frames() {
    let mut flow = rf_model::Flowsheet::new("large-browser");
    for index in 0..512 {
        flow.insert_stream(rf_model::MaterialStreamState::new(
            format!("s{index}"),
            format!("Stream {index}"),
        ))
        .unwrap();
    }
    for index in 0..384 {
        flow.insert_unit(rf_model::UnitNode::new(
            format!("u{index}"),
            format!("Unit {index}"),
            "heater",
            vec![],
        ))
        .unwrap();
    }
    let app = rf_ui::AppState::new(rf_ui::FlowsheetDocument::new(
        flow,
        rf_ui::DocumentMetadata::new("large", "Large", UNIX_EPOCH),
    ));
    let mut state = VariableBrowserState::default();
    let started = Instant::now();
    state.synchronize(&app.workspace, StudioShellLocale::En);
    let cold = started.elapsed();
    let started = Instant::now();
    for _ in 0..100 {
        state.synchronize(&app.workspace, StudioShellLocale::En);
    }
    let warm = started.elapsed();
    assert_eq!(state.cache.rebuilds, 1);
    assert_eq!(state.cache.objects.len(), 896);
    state
        .cache
        .filter("temperature", SectionFilter::Inputs, None);
    assert!(!state.cache.visible.is_empty());
    eprintln!(
        "browser metadata: {} objects, {} entries, cold {cold:?}, 100 warm frames {warm:?}",
        state.cache.objects.len(),
        state.cache.entries.len()
    );
}

#[test]
fn browser_failed_run_never_copies_result_or_template_as_result() {
    let mut config = super::super::tests::synced_workspace_config();
    config.project_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets/failures/unbound-inlet-port.rfproj.json");
    let mut app = super::super::tests::ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("run_panel.run_manual");
    synchronize(&mut app);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_result()
            .status(),
        rf_ui::result_query::ResultStatus::Failed
    );
    let selected = app
        .variable_browser
        .cache
        .entries
        .iter()
        .find(|e| e.section == RowSection::Result && e.target.field.is_some())
        .unwrap()
        .target
        .clone();
    let read = read_selection(
        &app.platform_host.modeling_inputs().workspace,
        &selected,
        app.locale,
    )
    .unwrap();
    assert_eq!(read.status, "Failed");
    assert!(read.copy.is_none());
    assert!(read.canonical.is_none());
    assert!(!read.source.is_empty());
    assert!(app.browser_copy_value(&selected).is_err());
}

#[test]
fn browser_distinguishes_zero_missing_unspecified_unsupported_and_conversion_failure() {
    let mut flow = rf_model::Flowsheet::new("states");
    flow.insert_component(rf_model::Component::new("a", "A"))
        .unwrap();
    let mut stream = rf_model::MaterialStreamState::new("s", "Stream");
    stream.total_molar_flow_mol_s = 0.;
    flow.insert_stream(stream).unwrap();
    flow.insert_unit(rf_model::UnitNode::new(
        "u",
        "Heater",
        "heater",
        vec![rf_model::UnitPort::new(
            "outlet",
            rf_types::PortDirection::Outlet,
            rf_types::PortKind::Material,
            Some("s".into()),
        )],
    ))
    .unwrap();
    let mut app = rf_ui::AppState::new(rf_ui::FlowsheetDocument::new(
        flow,
        rf_ui::DocumentMetadata::new("states", "States", UNIX_EPOCH),
    ));
    let reference = ProjectQuery::new(&app.workspace)
        .reference(ProjectObjectId::Flow(ObjectId::Stream("s".into())));
    let mut selection = BrowserSelection {
        object: reference,
        field: Some(ProjectField::Flow {
            section: VariableSection::Inputs,
            field: VariableField::MolarFlow,
        }),
    };
    let read = read_selection(&app.workspace, &selection, StudioShellLocale::En).unwrap();
    assert!(read.value.starts_with('0'));
    assert!(read.copy.is_some());
    assert!(read.source.contains("template"));
    selection.field = Some(ProjectField::Flow {
        section: VariableSection::Inputs,
        field: VariableField::MoleFraction("a".into()),
    });
    assert_eq!(
        read_selection(&app.workspace, &selection, StudioShellLocale::En)
            .unwrap()
            .status,
        "Unspecified"
    );
    selection.field = Some(ProjectField::Flow {
        section: VariableSection::Results,
        field: VariableField::Temperature,
    });
    assert_eq!(
        read_selection(&app.workspace, &selection, StudioShellLocale::En)
            .unwrap()
            .status,
        "Missing"
    );
    selection.field = Some(ProjectField::BasisValue);
    assert!(read_selection(&app.workspace, &selection, StudioShellLocale::En).is_err());
    selection.field = Some(ProjectField::Flow {
        section: VariableSection::Inputs,
        field: VariableField::MolarFlow,
    });
    // Public test fixture mutation: an otherwise finite SI value whose display conversion overflows.
    app.workspace
        .document
        .flowsheet
        .streams
        .get_mut(&StreamId::new("s"))
        .unwrap()
        .total_molar_flow_mol_s = f64::MAX;
    app.workspace
        .project_presentation
        .apply(rf_ui::ProjectPresentationCommand::Apply(
            rf_types::units::DisplayUnitSet::engineering(),
        ));
    let read = read_selection(&app.workspace, &selection, StudioShellLocale::En).unwrap();
    assert_eq!(read.value, "Conversion failed");
    assert!(read.copy.is_none());
}

#[test]
fn browser_rename_delete_and_undo_preserve_typed_selection_without_name_fallback() {
    let mut flow = rf_model::Flowsheet::new("identity");
    for id in ["first", "second"] {
        flow.insert_unit(rf_model::UnitNode::new(id, "Same name", "heater", vec![]))
            .unwrap();
    }
    let mut app = rf_ui::AppState::new(rf_ui::FlowsheetDocument::new(
        flow,
        rf_ui::DocumentMetadata::new("identity", "Identity", UNIX_EPOCH),
    ));
    let first = UnitId::new("first");
    let selection = BrowserSelection {
        object: ProjectQuery::new(&app.workspace)
            .reference(ProjectObjectId::Flow(ObjectId::Unit(first.clone()))),
        field: Some(ProjectField::Flow {
            section: VariableSection::Inputs,
            field: VariableField::Name,
        }),
    };
    let mut state = VariableBrowserState {
        selected: Some(selection.clone()),
        ..Default::default()
    };
    state.synchronize(&app.workspace, StudioShellLocale::En);
    app.write_variable(
        rf_ui::variable_commands::VariableWriteRequest {
            variable: VariableId {
                document: selection.object.document.clone(),
                object: ObjectId::Unit(first.clone()),
                section: VariableSection::Inputs,
                field: VariableField::Name,
            },
            expected_revision: app.workspace.document.revision,
            value: rf_ui::variable_browser::VariableValue::Text("Renamed".into()),
        },
        UNIX_EPOCH,
    )
    .unwrap();
    state.synchronize(&app.workspace, StudioShellLocale::En);
    assert_eq!(state.selected, Some(selection.clone()));
    assert_eq!(
        read_selection(&app.workspace, &selection, StudioShellLocale::En)
            .unwrap()
            .value,
        "Renamed"
    );
    app.delete_unit(&first, UNIX_EPOCH).unwrap();
    state.synchronize(&app.workspace, StudioShellLocale::En);
    assert!(read_selection(&app.workspace, &selection, StudioShellLocale::En).is_err());
    assert_eq!(state.selected, Some(selection.clone()));
    app.undo_document_command(UNIX_EPOCH).unwrap();
    state.synchronize(&app.workspace, StudioShellLocale::En);
    assert_eq!(
        read_selection(&app.workspace, &selection, StudioShellLocale::En)
            .unwrap()
            .value,
        "Renamed"
    );
}

#[test]
fn browser_rows_expose_keyboard_and_accessibility_identity() {
    let mut app = app();
    app.locale = StudioShellLocale::En;
    app.open_variable_browser();
    synchronize(&mut app);
    app.variable_browser.query = "heater-1 outlet temperature".into();
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let output = frame(&mut app, &ctx, 400., vec![]);
    let nodes = output.platform_output.accesskit_update.unwrap().nodes;
    assert!(nodes.iter().any(|(_, node)| {
        node.label()
            .is_some_and(|label| label.contains("Outlet temperature"))
    }));
    let before = app.platform_host.document().revision;
    // Tab through real controls; Enter activates a row, never a model action.
    for _ in 0..20 {
        frame(
            &mut app,
            &ctx,
            400.,
            vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        frame(
            &mut app,
            &ctx,
            400.,
            vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        if ctx.memory(|m| m.focused()).is_some() {
            let output = frame(&mut app, &ctx, 400., vec![]);
            let update = output.platform_output.accesskit_update.unwrap();
            if update.nodes.iter().any(|(id, node)| {
                *id == update.focus
                    && node
                        .label()
                        .is_some_and(|label| label.contains("Outlet temperature"))
            }) {
                frame(
                    &mut app,
                    &ctx,
                    400.,
                    vec![egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                );
                assert!(app.variable_browser.detail_page);
                assert_eq!(app.platform_host.document().revision, before);
                return;
            }
        }
    }
    panic!("result row was not reachable using Tab");
}

fn text_rects(output: &egui::FullOutput) -> Vec<(String, egui::Rect)> {
    fn collect(shape: &egui::Shape, rows: &mut Vec<(String, egui::Rect)>) {
        match shape {
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, rows);
                }
            }
            egui::Shape::Text(text) => rows.push((
                text.galley.job.text.clone(),
                text.galley.rect.translate(text.pos.to_vec2()),
            )),
            _ => {}
        }
    }
    let mut rows = Vec::new();
    for shape in &output.shapes {
        collect(&shape.shape, &mut rows);
    }
    rows
}

#[test]
fn browser_table_columns_align_and_clicking_value_selects_the_row() {
    for locale in [StudioShellLocale::En, StudioShellLocale::ZhCn] {
        let mut app = app();
        app.locale = locale;
        app.open_variable_browser();
        synchronize(&mut app);
        app.variable_browser.query = "heater-1 outlet temperature".into();
        let ctx = egui::Context::default();
        super::super::fonts::configure_studio_fonts(&ctx);
        frame(&mut app, &ctx, 1280., vec![]);
        let output = frame(&mut app, &ctx, 1280., vec![]);
        let text = text_rects(&output);
        let field = text
            .iter()
            .find(|(s, _)| s == tr(locale, "出口温度", "Outlet temperature"))
            .unwrap()
            .1;
        let value = text.iter().find(|(s, _)| s == "345 K").unwrap().1;
        let header = text
            .iter()
            .find(|(s, _)| s == tr(locale, "值 / 单位", "Value / unit"))
            .unwrap()
            .1;
        assert!(
            value.left() > field.right(),
            "numeric column must not collapse into the field label"
        );
        assert!(
            (value.left() - header.left()).abs() < 12.0,
            "column header must align with rows"
        );
        let pos = value.center();
        frame(
            &mut app,
            &ctx,
            1280.,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame(
            &mut app,
            &ctx,
            1280.,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(app.variable_browser.detail_page);
        assert_eq!(
            app.variable_browser.selected.as_ref().unwrap().field,
            Some(ProjectField::Flow {
                section: VariableSection::Inputs,
                field: VariableField::OutletTemperature
            })
        );
    }
}

#[test]
fn browser_wrapped_long_names_fit_narrow_rows_at_double_font() {
    let mut app = app();
    app.locale = StudioShellLocale::En;
    let name =
        "这是很长的设备名称 · A long equipment name with several words and a Unicode suffix αβγδ";
    super::super::tests::commit_unit_parameter(&mut app, "heater-1", "unit:heater-1:name", name);
    app.open_variable_browser();
    synchronize(&mut app);
    app.variable_browser.query = "heater-1 outlet temperature".into();
    let ctx = egui::Context::default();
    super::super::fonts::configure_studio_fonts(&ctx);
    ctx.style_mut(|s| {
        for font in s.text_styles.values_mut() {
            font.size *= 2.;
        }
    });
    frame(&mut app, &ctx, 400., vec![]);
    let output = frame(&mut app, &ctx, 400., vec![]);
    let text = text_rects(&output);
    let name_rect = text.iter().find(|(s, _)| s == name).unwrap().1;
    let value = text.iter().find(|(s, _)| s.starts_with("345 K")).unwrap().1;
    assert!(name_rect.height() > ctx.style().text_styles[&egui::TextStyle::Body].size);
    assert!(name_rect.right() < 400.0);
    assert!(name_rect.bottom() < value.top());
    assert!(value.right() < 400.0);
}

#[test]
fn browser_keeps_equipment_case_identity_when_membership_and_selection_change() {
    use rf_model::equipment::SeparatorRatingInput;
    use rf_ui::engineering::EngineeringCommand;
    let mut app = engineering_app();
    let task = rf_types::EquipmentTaskId::new("rating-1");
    let mut second = app
        .workspace
        .engineering
        .inputs()
        .rating_task(&task)
        .unwrap()
        .cases[0]
        .clone();
    second.id = "second-case".into();
    second
        .inputs
        .remove(&SeparatorRatingInput::RequiredWorkingTime);
    app.workspace
        .engineering
        .apply(vec![
            EngineeringCommand::UpgradeRatingMethod(task.clone()),
            EngineeringCommand::AddRatingCase {
                task: task.clone(),
                case: Box::new(second),
            },
        ])
        .unwrap();
    let query = ProjectQuery::new(&app.workspace);
    let field = Some(ProjectField::RatingInput(
        SeparatorRatingInput::RequiredWorkingTime,
    ));
    let first = BrowserSelection {
        object: query.reference(ProjectObjectId::Case(
            rf_ui::engineering::EquipmentCaseKey {
                task_id: task.clone(),
                case_id: "rating-case-1".into(),
            },
        )),
        field: field.clone(),
    };
    let second = BrowserSelection {
        object: query.reference(ProjectObjectId::Case(
            rf_ui::engineering::EquipmentCaseKey {
                task_id: task.clone(),
                case_id: "second-case".into(),
            },
        )),
        field,
    };
    let mut state = VariableBrowserState {
        selected: Some(first.clone()),
        ..Default::default()
    };
    state.synchronize(&app.workspace, StudioShellLocale::En);
    assert_eq!(
        read_selection(&app.workspace, &first, StudioShellLocale::En)
            .unwrap()
            .status,
        "Specified"
    );
    assert_eq!(
        read_selection(&app.workspace, &second, StudioShellLocale::En)
            .unwrap()
            .status,
        "Unspecified"
    );
    app.workspace
        .engineering
        .apply(vec![EngineeringCommand::ReorderRatingCases {
            task,
            cases: vec!["second-case".into(), "rating-case-1".into()],
        }])
        .unwrap();
    state.synchronize(&app.workspace, StudioShellLocale::En);
    assert_eq!(state.selected, Some(first.clone()));
    assert!(
        read_selection(&app.workspace, &first, StudioShellLocale::En)
            .unwrap()
            .canonical
            .unwrap()
            .contains("120")
    );
    assert_eq!(
        state
            .cache
            .objects
            .iter()
            .filter(|o| matches!(o.reference.object, ProjectObjectId::Geometry(_)))
            .count(),
        1
    );
}

#[test]
fn browser_side_owns_keys_only_inside_its_area() {
    let ctx = egui::Context::default();
    let state = VariableBrowserState {
        open: true,
        mode: BrowserMode::Side,
        area_rect: Some(egui::Rect::from_min_max(
            egui::pos2(800., 0.),
            egui::pos2(1200., 800.),
        )),
        ..Default::default()
    };
    for (pointer, owns) in [
        (egui::pos2(100., 100.), false),
        (egui::pos2(1000., 100.), true),
    ] {
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::PointerMoved(pointer)],
                ..Default::default()
            },
            |ctx| assert_eq!(state.owns_shortcuts(ctx), owns),
        );
    }
}
