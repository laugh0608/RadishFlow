use super::*;

fn fixture() -> rf_model::Flowsheet {
    let project: rf_store::StoredProjectFile = serde_json::from_str(include_str!(
        "../../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    project.document.flowsheet
}

#[test]
fn home_preview_bounds_projection_at_six_units() {
    let mut flowsheet = fixture();
    let mut unit = flowsheet.units.values().next().unwrap().clone();
    unit.ports.clear();
    for count in [0, 6, 7, 1536] {
        flowsheet.units.clear();
        for index in 0..count {
            let id = rf_types::UnitId::new(format!("unit-{index}"));
            unit.id = id.clone();
            flowsheet.units.insert(id, unit.clone());
        }
        let preview = preview_from_flowsheet(&flowsheet);
        if count == 6 {
            assert!(matches!(preview, HomePreview::SmallFlow { nodes, .. } if nodes.len() == 6));
        } else {
            assert_eq!(
                preview,
                HomePreview::Summary {
                    unit_count: count,
                    stream_count: Some(flowsheet.streams.len())
                }
            );
        }
    }
}

#[test]
fn home_preview_projects_actual_connections_in_direction_order() {
    let flowsheet = fixture();
    let HomePreview::SmallFlow {
        nodes,
        edges,
        stream_count,
    } = preview_from_flowsheet(&flowsheet)
    else {
        panic!()
    };
    let order = single_chain(nodes.len(), &edges).unwrap();
    let names = order
        .into_iter()
        .map(|i| nodes[i].as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["Feed", "Heater", "Flash Drum"]);
    assert_eq!(stream_count, Some(flowsheet.streams.len()));
}

#[test]
fn home_preview_never_chains_branches_cycles_or_disconnected_units() {
    for (count, edges) in [
        (3, vec![(0, 1), (0, 2)]),
        (3, vec![(0, 2), (1, 2)]),
        (3, vec![(0, 1), (1, 2), (2, 0)]),
        (4, vec![(0, 1), (2, 3)]),
        (4, vec![(0, 1), (2, 3), (3, 2)]),
        (3, vec![(0, 1), (0, 1)]),
        (2, vec![(0, 2)]),
        (1, vec![(0, 0)]),
    ] {
        assert_eq!(single_chain(count, &edges), None, "{edges:?}");
    }
    assert_eq!(
        single_chain(4, &[(3, 1), (0, 2), (1, 0)]),
        Some(vec![3, 1, 0, 2])
    );
}

fn render(
    preview: &HomePreview,
    width: f32,
    font_size: f32,
    locale: StudioShellLocale,
) -> (egui::Rect, egui::Rect, Vec<String>) {
    let ctx = egui::Context::default();
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::proportional(font_size),
    );
    ctx.set_style(style);
    let mut preview_rect = egui::Rect::NOTHING;
    let mut next_rect = egui::Rect::NOTHING;
    let mut output = None;
    // A settled second frame gives the same layout contract as the real dashboard.
    for _ in 0..2 {
        output = Some(ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 500.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    preview_rect = render_home_preview(ui, preview, locale).rect;
                    next_rect = ui.button("Next project").rect;
                });
            },
        ));
    }
    fn collect(shape: &egui::Shape, texts: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(text) => {
                assert!(text.galley.rows.len() <= 1, "preview label wrapped");
                texts.push(text.galley.job.text.clone());
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, texts);
                }
            }
            _ => {}
        }
    }
    let mut texts = Vec::new();
    for shape in output.unwrap().shapes {
        collect(&shape.shape, &mut texts);
    }
    (preview_rect, next_rect, texts)
}

#[test]
fn home_preview_stays_bounded_and_next_card_reachable_for_long_names_and_large_counts() {
    for locale in [StudioShellLocale::En, StudioShellLocale::ZhCn] {
        for width in [180.0, 840.0] {
            for font_size in [12.0, 28.0] {
                for preview in [
                    HomePreview::SmallFlow {
                        nodes: vec!["超长单元名称 Long equipment name ".repeat(40); 6],
                        edges: (0..5).map(|i| (i, i + 1)).collect(),
                        stream_count: Some(8),
                    },
                    HomePreview::Summary {
                        unit_count: 1536,
                        stream_count: Some(1536),
                    },
                    HomePreview::Unavailable,
                ] {
                    let (rect, next, _) = render(&preview, width, font_size, locale);
                    assert!(rect.width() <= width - 16.0 + 0.1, "{rect:?}, {width}");
                    assert!(rect.height() <= 2.0 * font_size * 1.5 + 18.0, "{rect:?}");
                    assert!(next.top() >= rect.bottom());
                    assert!(next.bottom() < 180.0, "next card unreachable: {next:?}");
                }
            }
        }
    }
}

#[test]
fn home_preview_shows_arrows_only_for_chain_and_counts_when_narrow() {
    let chain = HomePreview::SmallFlow {
        nodes: vec!["Feed".into(), "Heater".into(), "Flash".into()],
        edges: vec![(0, 1), (1, 2)],
        stream_count: Some(4),
    };
    let (_, _, texts) = render(&chain, 840.0, 12.0, StudioShellLocale::En);
    assert!(texts.iter().any(|text| text == "Flow connections"));
    assert_eq!(texts.iter().filter(|text| text.as_str() == "→").count(), 2);
    let (_, _, texts) = render(&chain, 180.0, 28.0, StudioShellLocale::En);
    assert!(texts.iter().any(|text| text == "3 units · 4 streams"));
    assert!(!texts.iter().any(|text| text == "→"));
    for edges in [vec![(0, 1), (0, 2)], vec![(0, 1), (1, 2), (2, 0)], vec![]] {
        let preview = HomePreview::SmallFlow {
            nodes: vec!["A".into(), "B".into(), "C".into()],
            edges,
            stream_count: Some(3),
        };
        let (_, _, texts) = render(&preview, 840.0, 12.0, StudioShellLocale::ZhCn);
        assert!(texts.iter().any(|text| text == "单元概览"));
        assert!(!texts.iter().any(|text| text == "→"));
    }
}
