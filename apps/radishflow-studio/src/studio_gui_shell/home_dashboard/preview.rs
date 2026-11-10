use std::collections::{BTreeMap, BTreeSet};

use eframe::egui;

use radishflow_studio::StudioGuiWindowHomePreviewModel as HomePreview;

use super::{StudioShellLocale, non_empty_trimmed};

pub(super) fn current_workspace_preview(
    document: &radishflow_studio::StudioGuiWorkspaceDocumentSnapshot,
) -> HomePreview {
    HomePreview::Summary {
        unit_count: document.unit_count,
        stream_count: Some(document.stream_count),
    }
}

pub(super) fn preview_from_flowsheet(flowsheet: &rf_model::Flowsheet) -> HomePreview {
    let unit_count = flowsheet.units.len();
    let stream_count = Some(flowsheet.streams.len());
    // Bound the projection before visiting names or port bindings on large projects.
    if unit_count == 0 || unit_count > HomePreview::MAX_UNITS {
        return HomePreview::Summary {
            unit_count,
            stream_count,
        };
    }
    let mut nodes = Vec::with_capacity(unit_count);
    let mut sources = BTreeMap::<_, BTreeSet<usize>>::new();
    let mut sinks = BTreeMap::<_, BTreeSet<usize>>::new();
    for (index, unit) in flowsheet.units.values().enumerate() {
        nodes.push(non_empty_trimmed(&unit.name).unwrap_or_else(|| readable_unit_kind(&unit.kind)));
        for port in &unit.ports {
            if port.kind != rf_types::PortKind::Material {
                continue;
            }
            let Some(stream_id) = port.stream_id.as_ref() else {
                continue;
            };
            let endpoints = match port.direction {
                rf_types::PortDirection::Outlet => &mut sources,
                rf_types::PortDirection::Inlet => &mut sinks,
            };
            endpoints.entry(stream_id).or_default().insert(index);
        }
    }
    let mut edges = BTreeSet::new();
    for (stream, from_units) in sources {
        if let Some(to_units) = sinks.get(stream) {
            for from in from_units {
                for to in to_units {
                    edges.insert((from, *to));
                }
            }
        }
    }
    HomePreview::SmallFlow {
        nodes,
        edges: edges.into_iter().collect(),
        stream_count,
    }
}

fn readable_unit_kind(kind: &str) -> String {
    let label = kind
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if label.is_empty() {
        "Unit".to_string()
    } else {
        label
    }
}

// Only a connected directed path can be represented by a row of arrows.
fn single_chain(node_count: usize, edges: &[(usize, usize)]) -> Option<Vec<usize>> {
    if node_count < 2 || edges.len() != node_count - 1 {
        return None;
    }
    let mut incoming = vec![false; node_count];
    let mut next = vec![None; node_count];
    for &(from, to) in edges {
        if from >= node_count
            || to >= node_count
            || from == to
            || incoming[to]
            || next[from].is_some()
        {
            return None;
        }
        incoming[to] = true;
        next[from] = Some(to);
    }
    let mut heads = (0..node_count).filter(|&i| !incoming[i]);
    let head = heads.next()?;
    if heads.next().is_some() {
        return None;
    }
    let mut order = vec![head];
    while let Some(to) = next[*order.last()?] {
        if order.len() >= node_count {
            return None;
        }
        order.push(to);
    }
    (order.len() == node_count).then_some(order)
}

pub(super) fn render_home_preview(
    ui: &mut egui::Ui,
    preview: &HomePreview,
    locale: StudioShellLocale,
) -> egui::Response {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(244, 247, 251))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(218, 226, 238),
        ))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            match preview {
                HomePreview::Unavailable => {
                    preview_line(ui, localized(locale, "Preview unavailable", "预览不可用"));
                    preview_line(
                        ui,
                        localized(locale, "Open project to inspect", "打开工程查看"),
                    );
                }
                HomePreview::Summary {
                    unit_count,
                    stream_count,
                } => {
                    render_summary(ui, *unit_count, *stream_count, locale);
                }
                HomePreview::SmallFlow {
                    nodes,
                    edges,
                    stream_count,
                } => {
                    render_small_flow(ui, nodes, edges, *stream_count, locale);
                }
            }
        })
        .response
}

fn render_small_flow(
    ui: &mut egui::Ui,
    nodes: &[String],
    edges: &[(usize, usize)],
    stream_count: Option<usize>,
    locale: StudioShellLocale,
) {
    if nodes.is_empty() || nodes.len() > HomePreview::MAX_UNITS {
        render_summary(ui, nodes.len(), stream_count, locale);
        return;
    }
    let chain = single_chain(nodes.len(), edges);
    let font = egui::TextStyle::Small.resolve(ui.style());
    let arrow_width = ui
        .painter()
        .layout_no_wrap("→".into(), font.clone(), ui.visuals().text_color())
        .size()
        .x;
    let gap = ui.spacing().item_spacing.x;
    let separators = (nodes.len() - 1) as f32;
    let spacing = separators
        * if chain.is_some() {
            arrow_width + 2.0 * gap
        } else {
            gap
        };
    let chip_width = (ui.available_width() - spacing) / nodes.len() as f32;
    // Keep at least four ems for a readable label; narrow cards use counts instead.
    if chip_width < font.size * 4.0 + 8.0 {
        render_summary(ui, nodes.len(), stream_count, locale);
        return;
    }
    preview_line(
        ui,
        if chain.is_some() {
            localized(locale, "Flow connections", "流程连接")
        } else {
            localized(locale, "Unit overview", "单元概览")
        },
    );
    let order = chain
        .as_deref()
        .map_or_else(|| (0..nodes.len()).collect(), |order| order.to_vec());
    let height = ui.text_style_height(&egui::TextStyle::Small);
    ui.horizontal(|ui| {
        for (position, index) in order.into_iter().enumerate() {
            if position > 0 && chain.is_some() {
                ui.add_sized(
                    [arrow_width, height],
                    egui::Label::new(egui::RichText::new("→").small()).truncate(),
                );
            }
            let name = &nodes[index];
            let response = ui.add_sized(
                [chip_width, height],
                egui::Label::new(
                    egui::RichText::new(name)
                        .small()
                        .color(egui::Color32::from_rgb(52, 83, 133))
                        .background_color(egui::Color32::from_rgb(226, 234, 247)),
                )
                .truncate(),
            );
            response.on_hover_text(name);
        }
    });
}

fn render_summary(
    ui: &mut egui::Ui,
    unit_count: usize,
    stream_count: Option<usize>,
    locale: StudioShellLocale,
) {
    let counts = match (locale, stream_count) {
        (StudioShellLocale::En, Some(streams)) => format!("{unit_count} units · {streams} streams"),
        (StudioShellLocale::ZhCn, Some(streams)) => {
            format!("{unit_count} 个单元 · {streams} 条物流")
        }
        (StudioShellLocale::En, None) => format!("{unit_count} units"),
        (StudioShellLocale::ZhCn, None) => format!("{unit_count} 个单元"),
    };
    preview_line(ui, &counts);
    preview_line(
        ui,
        localized(
            locale,
            "Open project to view the full flowsheet",
            "打开工程查看完整流程",
        ),
    );
}

fn preview_line(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(egui::RichText::new(text).small()).truncate())
        .on_hover_text(text);
}

fn localized(locale: StudioShellLocale, en: &'static str, zh: &'static str) -> &'static str {
    match locale {
        StudioShellLocale::En => en,
        StudioShellLocale::ZhCn => zh,
    }
}

#[cfg(test)]
mod tests;
