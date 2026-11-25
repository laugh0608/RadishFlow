//! Transient port markers for the Inspector's reconnect picker.
use super::*;
use crate::studio_gui_shell::stream_reconnect::ReconnectCanvasPorts;
use rf_types::PortDirection;
use rf_ui::StreamPortBinding;
use std::collections::BTreeMap;

const PREVIEW: egui::Color32 = egui::Color32::from_rgb(112, 58, 161);
const FIXED: egui::Color32 = egui::Color32::from_rgb(23, 91, 100);

struct CandidateMarker {
    anchor: egui::Pos2,
    chosen: bool,
    highlighted: bool,
}

pub(super) struct ReconnectOverlay {
    fixed: egui::Pos2,
    candidates: Vec<CandidateMarker>,
    preview: Option<Vec<egui::Pos2>>,
}

impl ReconnectOverlay {
    pub(super) fn project(
        rect: egui::Rect,
        transform: &CanvasViewportTransform,
        units: &[radishflow_studio::StudioGuiCanvasUnitBlockViewModel],
        drag: Option<&CanvasUnitDragState>,
        ports: ReconnectCanvasPorts<'_>,
    ) -> Option<Self> {
        // Resolve the picker identities against the same rectangles and side slots that
        // draw the equipment, including a live layout drag and the current viewport.
        let layout: BTreeMap<_, _> = units
            .iter()
            .map(|unit| {
                let bounds = canvas_displayed_unit_rect(rect, transform, unit, drag);
                (unit.unit_id.as_str(), (unit, bounds))
            })
            .collect();
        let anchor = |binding: &StreamPortBinding, direction: PortDirection| {
            let (unit, rect) = layout.get(binding.unit_id.as_str())?;
            let port = unit
                .ports
                .iter()
                .find(|p| p.name == binding.port && p.direction_label == direction.as_str())?;
            Some(canvas_unit_port_anchor_in_rect(
                *rect,
                direction == PortDirection::Outlet,
                port.side_index,
                port.side_count,
            ))
        };
        let missing = ports.options.missing;
        let fixed = anchor(&ports.options.fixed, missing.opposite())?;
        let candidates = ports
            .options
            .candidates
            .iter()
            .filter_map(|candidate| {
                Some(CandidateMarker {
                    anchor: anchor(candidate, missing)?,
                    chosen: ports.chosen == Some(candidate),
                    highlighted: ports.highlighted == Some(candidate),
                })
            })
            .collect::<Vec<_>>();
        let preview = candidates
            .iter()
            .find(|marker| marker.chosen)
            .map(|marker| {
                let chosen = marker.anchor;
                let (start, end) = match missing {
                    PortDirection::Inlet => (fixed, chosen),
                    PortDirection::Outlet => (chosen, fixed),
                };
                preview_path(start, end, layout.values().map(|(_, rect)| *rect))
            });
        Some(Self {
            fixed,
            candidates,
            preview,
        })
    }

    pub(super) fn paint(&self, painter: &egui::Painter) {
        if let Some(path @ [.., last, end]) = self.preview.as_deref() {
            for segment in path.windows(2) {
                let geometry = CanvasStreamLineGeometry {
                    start: segment[0],
                    end: segment[1],
                };
                if let Some(visible) = streams::clip_stream_segment(geometry, painter.clip_rect()) {
                    painter.extend(egui::Shape::dashed_line(
                        &visible,
                        egui::Stroke::new(2.0, PREVIEW),
                        6.0,
                        4.0,
                    ));
                }
            }
            let delta = *end - *last;
            let arrow_end = *end - delta.normalized() * 7.0_f32.min(delta.length() * 0.3);
            streams::paint_canvas_stream_arrow(
                painter,
                CanvasStreamLineGeometry {
                    start: *last,
                    end: arrow_end,
                },
                PREVIEW,
            );
        }
        let diamond = [
            egui::vec2(0.0, -7.0),
            egui::vec2(7.0, 0.0),
            egui::vec2(0.0, 7.0),
            egui::vec2(-7.0, 0.0),
        ]
        .map(|offset| self.fixed + offset);
        painter.add(egui::Shape::closed_line(
            diamond.to_vec(),
            egui::Stroke::new(2.0, FIXED),
        ));
        for &CandidateMarker {
            anchor,
            chosen,
            highlighted,
        } in &self.candidates
        {
            painter.circle(
                anchor,
                5.5,
                if chosen {
                    PREVIEW
                } else {
                    egui::Color32::WHITE
                },
                egui::Stroke::new(1.8, PREVIEW),
            );
            if highlighted {
                painter.rect_stroke(
                    egui::Rect::from_center_size(anchor, egui::vec2(15.0, 15.0)),
                    2.0,
                    egui::Stroke::new(1.6, PREVIEW),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
}

// Only the temporary hint takes a detour. Its source and sink stay on the real
// port anchors even when the two devices sit on opposite sides of a third one.
fn preview_path(
    start: egui::Pos2,
    end: egui::Pos2,
    blocks: impl Iterator<Item = egui::Rect> + Clone,
) -> Vec<egui::Pos2> {
    let direct = CanvasStreamLineGeometry { start, end };
    if end.x > start.x
        && !blocks.clone().any(|block| {
            !block.contains(start)
                && !block.contains(end)
                && streams::clip_stream_segment(direct, block).is_some()
        })
    {
        return vec![start, end];
    }
    let leave = start + egui::vec2(10.0, 0.0);
    let enter = end - egui::vec2(10.0, 0.0);
    let corridor = egui::Rect::from_two_pos(start, end);
    let mut span = corridor;
    for block in blocks.filter(|block| {
        block.left() <= leave.x.max(enter.x)
            && block.right() >= leave.x.min(enter.x)
            && block.top() <= corridor.bottom()
            && block.bottom() >= corridor.top()
    }) {
        span = span.union(block);
    }
    let y = if start.y + end.y <= span.top() + span.bottom() {
        span.top() - 10.0
    } else {
        span.bottom() + 10.0
    };
    vec![
        start,
        leave,
        egui::pos2(leave.x, y),
        egui::pos2(enter.x, y),
        enter,
        end,
    ]
}

#[cfg(test)]
mod tests;
