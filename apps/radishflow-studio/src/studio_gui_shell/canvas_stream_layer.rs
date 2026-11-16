//! Read-only stream coloring derived from the current flowsheet result.
use super::*;
use rf_types::units::{MeasurementUnit, QuantityKind};
use rf_ui::{
    StreamStateSnapshot,
    result_query::{FlowResultView, ResultStatus},
};
use std::collections::HashMap;

const LOW: egui::Color32 = egui::Color32::from_rgb(109, 129, 155);
const HIGH: egui::Color32 = egui::Color32::from_rgb(24, 73, 136);
const NEUTRAL: egui::Color32 = egui::Color32::from_rgb(99, 113, 127);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum CanvasStreamLayer {
    #[default]
    ConnectionDirection,
    MolarFlow,
    Temperature,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StreamColorVariable {
    MolarFlow,
    Temperature,
}

pub(super) struct CanvasStreamColoring<'a> {
    variable: StreamColorVariable,
    result: StreamColorResult<'a>,
}

enum StreamColorResult<'a> {
    Current {
        readings: HashMap<&'a str, StreamColorReading>,
        scale: Option<StreamColorScale>,
        unit: MeasurementUnit,
        sequence: u64,
        revision: u64,
    },
    Unavailable(ResultStatus),
}

#[derive(Clone, Copy)]
enum StreamColorReading {
    Value(f64),
    Missing,
    ZeroFlow,
    InvalidFlow,
    InvalidTemperature,
}

#[derive(Clone, Copy)]
struct StreamColorScale {
    minimum: f64,
    maximum: f64,
}

pub(super) struct StreamLayerStroke {
    pub(super) color: egui::Color32,
    pub(super) dashed: bool,
}

impl CanvasStreamLayer {
    pub(super) fn project(
        self,
        workspace: &rf_ui::WorkspaceState,
    ) -> Option<CanvasStreamColoring<'_>> {
        let variable = match self {
            Self::ConnectionDirection => return None,
            Self::MolarFlow => StreamColorVariable::MolarFlow,
            Self::Temperature => StreamColorVariable::Temperature,
        };
        let view = workspace.flow_result();
        let result = match view {
            FlowResultView::Current { snapshot, origin } => {
                let readings: HashMap<_, _> = snapshot
                    .streams
                    .iter()
                    .map(|stream| (stream.stream_id.as_str(), variable.reading(stream)))
                    .collect();
                let scale = StreamColorScale::from_readings(variable, readings.values().copied());
                StreamColorResult::Current {
                    readings,
                    scale,
                    unit: workspace
                        .project_presentation
                        .display_units()
                        .unit_for(variable.quantity()),
                    sequence: snapshot.sequence,
                    revision: origin.revision(),
                }
            }
            _ => StreamColorResult::Unavailable(view.status()),
        };
        Some(CanvasStreamColoring { variable, result })
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
            (
                Self::Temperature,
                localized(locale, "流股温度", "Stream temperature"),
            ),
        ] {
            let response = ui.radio_value(self, layer, label);
            if response.has_focus() {
                response.scroll_to_me(None);
            }
        }
    }
}

impl StreamColorVariable {
    fn quantity(self) -> QuantityKind {
        match self {
            Self::MolarFlow => QuantityKind::MolarFlow,
            Self::Temperature => QuantityKind::AbsoluteTemperature,
        }
    }

    fn reading(self, stream: &StreamStateSnapshot) -> StreamColorReading {
        let flow = stream.total_molar_flow_mol_s;
        match self {
            Self::MolarFlow if is_valid_result_molar_flow(flow) => StreamColorReading::Value(flow),
            Self::MolarFlow => StreamColorReading::Missing,
            Self::Temperature if !is_valid_result_molar_flow(flow) => {
                StreamColorReading::InvalidFlow
            }
            Self::Temperature if flow == 0.0 => StreamColorReading::ZeroFlow,
            Self::Temperature if stream.temperature_k.is_finite() && stream.temperature_k > 0.0 => {
                StreamColorReading::Value(stream.temperature_k)
            }
            Self::Temperature => StreamColorReading::InvalidTemperature,
        }
    }

    fn format(self, value: f64, unit: MeasurementUnit, locale: StudioShellLocale) -> String {
        rf_ui::format_quantity(value, self.quantity(), unit).unwrap_or_else(|_| {
            localized(
                locale,
                "无法以项目单位显示",
                "Cannot display in the project unit",
            )
            .into()
        })
    }
}

impl StreamColorScale {
    fn from_readings(
        variable: StreamColorVariable,
        readings: impl Iterator<Item = StreamColorReading>,
    ) -> Option<Self> {
        let mut values = readings.filter_map(|reading| match reading {
            StreamColorReading::Value(value) => Some(value),
            _ => None,
        });
        let first = values.next()?;
        let (minimum, maximum) = values.fold((first, first), |(low, high), value| {
            (low.min(value), high.max(value))
        });
        Some(Self {
            minimum: match variable {
                StreamColorVariable::MolarFlow => 0.0,
                StreamColorVariable::Temperature => minimum,
            },
            maximum,
        })
    }

    fn color(self, variable: StreamColorVariable, value: f64) -> egui::Color32 {
        let ratio = if self.minimum == self.maximum {
            match variable {
                StreamColorVariable::MolarFlow => 0.0,
                StreamColorVariable::Temperature => 0.5,
            }
        } else {
            (value - self.minimum) / (self.maximum - self.minimum)
        };
        scale_color(ratio)
    }
}

impl CanvasStreamColoring<'_> {
    pub(super) fn stroke(&self, stream_id: &str) -> StreamLayerStroke {
        let StreamColorResult::Current {
            readings, scale, ..
        } = &self.result
        else {
            return StreamLayerStroke {
                color: NEUTRAL,
                dashed: false,
            };
        };
        match (readings.get(stream_id), scale) {
            (Some(StreamColorReading::Value(value)), Some(scale)) => StreamLayerStroke {
                color: scale.color(self.variable, *value),
                dashed: false,
            },
            _ => StreamLayerStroke {
                color: NEUTRAL,
                dashed: true,
            },
        }
    }

    pub(super) fn stream_readout(
        &self,
        stream_id: &str,
        locale: StudioShellLocale,
    ) -> Option<String> {
        let StreamColorResult::Current { readings, unit, .. } = &self.result else {
            return None;
        };
        let reading = readings
            .get(stream_id)
            .copied()
            .unwrap_or(StreamColorReading::Missing);
        Some(match reading {
            StreamColorReading::Value(value) => self.variable.format(value, *unit, locale),
            StreamColorReading::ZeroFlow => localized(locale, "零流量", "Zero flow").into(),
            StreamColorReading::InvalidFlow => {
                localized(locale, "流量无效", "Invalid molar flow").into()
            }
            StreamColorReading::InvalidTemperature => {
                localized(locale, "温度无效", "Invalid temperature").into()
            }
            StreamColorReading::Missing => match self.variable {
                StreamColorVariable::MolarFlow => localized(locale, "无流量数据", "No flow value"),
                StreamColorVariable::Temperature => localized(locale, "温度缺失", "No temperature"),
            }
            .into(),
        })
    }

    pub(super) fn render_legend(
        &self,
        ui: &mut egui::Ui,
        locale: StudioShellLocale,
        streams: &[radishflow_studio::StudioGuiCanvasStreamLineViewModel],
    ) {
        ui.add_space(4.0);
        ui.strong(match self.variable {
            StreamColorVariable::MolarFlow => {
                localized(locale, "摩尔流量图例", "Molar flow legend")
            }
            StreamColorVariable::Temperature => {
                localized(locale, "流股温度图例", "Stream temperature legend")
            }
        });
        let (readings, scale, unit, sequence, revision) = match &self.result {
            StreamColorResult::Current {
                readings,
                scale,
                unit,
                sequence,
                revision,
            } => (readings, scale, unit, sequence, revision),
            StreamColorResult::Unavailable(status) => {
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
        render_wrapped_small(
            ui,
            format!(
                "{} · {} #{sequence} · r{revision}",
                localized(locale, "当前快照", "Current snapshot"),
                localized(locale, "运行", "Run"),
            ),
        );
        if let Some(scale) = scale {
            ui.horizontal_wrapped(|ui| {
                paint_scale(
                    ui,
                    scale.color(self.variable, scale.minimum),
                    scale.color(self.variable, scale.maximum),
                );
                let minimum = self.variable.format(scale.minimum, *unit, locale);
                let label = if self.variable == StreamColorVariable::Temperature
                    && scale.minimum == scale.maximum
                {
                    format!("{minimum} · {}", localized(locale, "等温", "Isothermal"))
                } else {
                    format!(
                        "{minimum} – {}",
                        self.variable.format(scale.maximum, *unit, locale)
                    )
                };
                render_wrapped_small(ui, label);
            });
        }
        let mut zero_flow = false;
        let mut unavailable = false;
        for stream in streams {
            match readings.get(stream.stream_id.as_str()) {
                Some(StreamColorReading::Value(_)) => {}
                Some(StreamColorReading::ZeroFlow) => zero_flow = true,
                _ => unavailable = true,
            }
        }
        if zero_flow {
            render_wrapped_small(
                ui,
                localized(locale, "虚线表示零流量。", "Dashes mark zero flow."),
            );
        }
        if unavailable || (scale.is_none() && !zero_flow) {
            render_wrapped_small(
                ui,
                match self.variable {
                    StreamColorVariable::MolarFlow => {
                        localized(locale, "虚线：无流量数据。", "Dashed line: no flow value.")
                    }
                    StreamColorVariable::Temperature => localized(
                        locale,
                        "虚线表示温度不可用。",
                        "Dashes mark unavailable temperatures.",
                    ),
                },
            );
        }
    }
}

pub(super) fn is_valid_result_molar_flow(amount: f64) -> bool {
    amount.is_finite() && amount >= 0.0
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

fn paint_scale(ui: &mut egui::Ui, left: egui::Color32, right: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(92.0, 9.0), egui::Sense::hover());
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), left);
    mesh.colored_vertex(rect.right_top(), right);
    mesh.colored_vertex(rect.right_bottom(), right);
    mesh.colored_vertex(rect.left_bottom(), left);
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
