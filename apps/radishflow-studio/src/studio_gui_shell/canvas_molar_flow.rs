//! Read-only projection of the current flowsheet result for canvas coloring.
use super::*;
use rf_types::units::{MeasurementUnit, QuantityKind};
use rf_ui::result_query::{FlowResultView, ResultStatus};
use std::collections::HashMap;

const LOW: egui::Color32 = egui::Color32::from_rgb(109, 129, 155);
const HIGH: egui::Color32 = egui::Color32::from_rgb(24, 73, 136);
const NEUTRAL: egui::Color32 = egui::Color32::from_rgb(99, 113, 127);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum CanvasStreamLayer {
    #[default]
    ConnectionDirection,
    MolarFlow,
}

pub(super) enum CanvasMolarFlow<'a> {
    Current {
        amounts: HashMap<&'a str, f64>,
        maximum: Option<f64>,
        unit: MeasurementUnit,
        sequence: u64,
        revision: u64,
    },
    Unavailable(ResultStatus),
}

pub(super) struct MolarFlowStroke {
    pub(super) color: egui::Color32,
    pub(super) dashed: bool,
}

impl CanvasStreamLayer {
    pub(super) fn project(self, workspace: &rf_ui::WorkspaceState) -> Option<CanvasMolarFlow<'_>> {
        if self == Self::ConnectionDirection {
            return None;
        }
        let view = workspace.flow_result();
        let FlowResultView::Current { snapshot, origin } = view else {
            return Some(CanvasMolarFlow::Unavailable(view.status()));
        };
        let amounts: HashMap<_, _> = snapshot
            .streams
            .iter()
            .filter_map(|stream| {
                let amount = stream.total_molar_flow_mol_s;
                is_valid_result_molar_flow(amount).then_some((stream.stream_id.as_str(), amount))
            })
            .collect();
        let maximum = amounts.values().copied().reduce(f64::max);
        Some(CanvasMolarFlow::Current {
            amounts,
            maximum,
            unit: workspace
                .project_presentation
                .display_units()
                .unit_for(QuantityKind::MolarFlow),
            sequence: snapshot.sequence,
            revision: origin.revision(),
        })
    }

    pub(super) fn render_choice(&mut self, ui: &mut egui::Ui, locale: StudioShellLocale) {
        ui.separator();
        ui.small(localized(locale, "图层", "Layer"));
        for (layer, label) in [
            (
                Self::ConnectionDirection,
                localized(locale, "连接方向", "Connection direction"),
            ),
            (Self::MolarFlow, localized(locale, "摩尔流量", "Molar flow")),
        ] {
            let response = ui.radio_value(self, layer, label);
            if response.has_focus() {
                response.scroll_to_me(None);
            }
        }
    }
}

impl CanvasMolarFlow<'_> {
    pub(super) fn stroke(&self, stream_id: &str) -> MolarFlowStroke {
        match self {
            Self::Current {
                amounts, maximum, ..
            } => match (amounts.get(stream_id), maximum) {
                (Some(amount), Some(maximum)) => MolarFlowStroke {
                    color: scale_color(if *maximum == 0.0 {
                        0.0
                    } else {
                        amount / maximum
                    }),
                    dashed: false,
                },
                _ => MolarFlowStroke {
                    color: NEUTRAL,
                    dashed: true,
                },
            },
            Self::Unavailable(_) => MolarFlowStroke {
                color: NEUTRAL,
                dashed: false,
            },
        }
    }

    pub(super) fn stream_readout(
        &self,
        stream_id: &str,
        locale: StudioShellLocale,
    ) -> Option<String> {
        let Self::Current { amounts, unit, .. } = self else {
            return None;
        };
        Some(match amounts.get(stream_id) {
            Some(&amount) => format_flow(amount, *unit, locale),
            None => localized(locale, "无流量数据", "No flow value").into(),
        })
    }

    pub(super) fn render_legend(
        &self,
        ui: &mut egui::Ui,
        locale: StudioShellLocale,
        streams: &[radishflow_studio::StudioGuiCanvasStreamLineViewModel],
    ) {
        ui.add_space(4.0);
        let title = localized(locale, "摩尔流量图例", "Molar flow legend");
        let (amounts, maximum, unit, sequence, revision) = match self {
            Self::Current {
                amounts,
                maximum,
                unit,
                sequence,
                revision,
            } => (amounts, maximum, unit, sequence, revision),
            Self::Unavailable(status) => {
                ui.strong(title);
                render_wrapped_small(
                    ui,
                    match status {
                        ResultStatus::Stale => localized(
                            locale,
                            "结果已过期，保留连接方向。",
                            "Stale result. Showing connection direction.",
                        ),
                        ResultStatus::Failed => localized(
                            locale,
                            "本次运行未生成可着色的结果，保留连接方向。",
                            "The run produced no result for coloring. Showing connection direction.",
                        ),
                        _ => localized(
                            locale,
                            "尚无当前结果，保留连接方向。",
                            "No current result. Showing connection direction.",
                        ),
                    },
                );
                return;
            }
        };

        ui.strong(title);
        render_wrapped_small(
            ui,
            format!(
                "{} · {} #{sequence} · r{revision}",
                localized(locale, "当前快照", "Current snapshot"),
                localized(locale, "运行", "Run"),
            ),
        );
        if let Some(maximum) = maximum {
            ui.horizontal_wrapped(|ui| {
                paint_scale(ui, *maximum > 0.0);
                render_wrapped_small(
                    ui,
                    format!(
                        "{} – {}",
                        format_flow(0.0, *unit, locale),
                        format_flow(*maximum, *unit, locale),
                    ),
                );
            });
        }
        if maximum.is_none()
            || streams
                .iter()
                .any(|line| !amounts.contains_key(line.stream_id.as_str()))
        {
            render_wrapped_small(
                ui,
                localized(locale, "虚线：无流量数据。", "Dashed line: no flow value."),
            );
        }
    }
}

pub(super) fn is_valid_result_molar_flow(amount: f64) -> bool {
    amount.is_finite() && amount >= 0.0
}

fn format_flow(amount: f64, unit: MeasurementUnit, locale: StudioShellLocale) -> String {
    rf_ui::format_quantity(amount, QuantityKind::MolarFlow, unit).unwrap_or_else(|_| {
        localized(
            locale,
            "无法以项目单位显示",
            "Cannot display in the project unit",
        )
        .into()
    })
}

fn scale_color(ratio: f64) -> egui::Color32 {
    let channel =
        |low: u8, high: u8| (low as f64 + ratio * (high as f64 - low as f64)).round() as u8;
    egui::Color32::from_rgb(
        channel(LOW.r(), HIGH.r()),
        channel(LOW.g(), HIGH.g()),
        channel(LOW.b(), HIGH.b()),
    )
}

fn paint_scale(ui: &mut egui::Ui, varying: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(92.0, 9.0), egui::Sense::hover());
    let right = if varying { HIGH } else { LOW };
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), LOW);
    mesh.colored_vertex(rect.right_top(), right);
    mesh.colored_vertex(rect.right_bottom(), right);
    mesh.colored_vertex(rect.left_bottom(), LOW);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(mesh);
}

fn localized(locale: StudioShellLocale, zh: &'static str, en: &'static str) -> &'static str {
    match locale {
        StudioShellLocale::ZhCn => zh,
        StudioShellLocale::En => en,
    }
}

#[cfg(test)]
mod tests;
