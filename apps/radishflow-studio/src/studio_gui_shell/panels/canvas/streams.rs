//! Material stream geometry annotations and presentation on the process canvas.
use super::*;
use crate::studio_gui_shell::canvas_stream_layer::StreamLayerStroke;

#[derive(Clone, Copy)]
pub(super) enum OpenPathEnd {
    Source,
    Sink,
}

impl OpenPathEnd {
    pub(super) fn shorten(self, geometry: &mut CanvasStreamLineGeometry) {
        let stub = egui::vec2(10.0, 0.0);
        match self {
            Self::Source => geometry.start = geometry.end - stub,
            Self::Sink => geometry.end = geometry.start + stub,
        }
    }
}

pub(super) fn paint_canvas_stream_line(
    painter: &egui::Painter,
    geometry: CanvasStreamLineGeometry,
    selected: bool,
    open_end: Option<OpenPathEnd>,
    flow_stroke: Option<StreamLayerStroke>,
) {
    let color = flow_stroke
        .as_ref()
        .map_or_else(|| topology_color(selected), |stroke| stroke.color);
    let width = if flow_stroke.is_some() || selected {
        2.4
    } else {
        1.6
    };
    let stroke = egui::Stroke::new(width, color);
    if selected {
        painter.line_segment(
            [geometry.start, geometry.end],
            egui::Stroke::new(
                if flow_stroke.is_some() { 8.0 } else { 6.0 },
                egui::Color32::from_rgba_unmultiplied(
                    48,
                    112,
                    188,
                    if flow_stroke.is_some() { 60 } else { 42 },
                ),
            ),
        );
    }
    painter.line_segment(
        [geometry.start, geometry.end],
        egui::Stroke::new(
            4.4_f32,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 210),
        ),
    );
    if flow_stroke.is_some_and(|stroke| stroke.dashed) {
        if let Some(visible) = clip_stream_segment(geometry, painter.clip_rect().expand(4.0)) {
            painter.extend(egui::Shape::dashed_line(&visible, stroke, 5.0, 4.0));
        }
    } else {
        painter.line_segment([geometry.start, geometry.end], stroke);
    }
    if !matches!(open_end, Some(OpenPathEnd::Source)) {
        painter.circle_filled(geometry.start, 3.5, color);
    }
    if let Some(open_end) = open_end {
        let position = match open_end {
            OpenPathEnd::Source => geometry.start,
            OpenPathEnd::Sink => geometry.end,
        };
        painter.circle(position, 3.5, egui::Color32::WHITE, stroke);
    } else {
        paint_canvas_stream_arrow(painter, geometry, color);
    }
}

fn topology_color(selected: bool) -> egui::Color32 {
    if selected {
        egui::Color32::from_rgb(32, 102, 176)
    } else {
        egui::Color32::from_rgb(42, 142, 122)
    }
}

// Subdivide only the visible portion. Stream endpoints can be arbitrarily far
// from the viewport on a large manually arranged flowsheet.
fn clip_stream_segment(
    geometry: CanvasStreamLineGeometry,
    clip: egui::Rect,
) -> Option<[egui::Pos2; 2]> {
    let delta = geometry.end - geometry.start;
    let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
    for (start, direction, low, high) in [
        (geometry.start.x, delta.x, clip.left(), clip.right()),
        (geometry.start.y, delta.y, clip.top(), clip.bottom()),
    ] {
        if direction == 0.0 {
            if start < low || start > high {
                return None;
            }
        } else {
            let a = (low - start) / direction;
            let b = (high - start) / direction;
            enter = enter.max(a.min(b));
            leave = leave.min(a.max(b));
        }
    }
    (enter < leave).then(|| {
        [
            geometry.start + enter * delta,
            geometry.start + leave * delta,
        ]
    })
}

pub(super) fn paint_flow_path_motion(
    painter: &egui::Painter,
    geometry: CanvasStreamLineGeometry,
    phase: f32,
) {
    for fraction in [phase, (phase + 0.5).fract()] {
        let position = geometry.start.lerp(geometry.end, fraction);
        painter.circle(
            position,
            3.5,
            egui::Color32::WHITE,
            egui::Stroke::new(1.5, egui::Color32::from_rgb(32, 102, 176)),
        );
    }
}

pub(super) fn paint_canvas_viewport_stream_focus(
    painter: &egui::Painter,
    geometry: CanvasStreamLineGeometry,
) {
    painter.line_segment(
        [geometry.start, geometry.end],
        egui::Stroke::new(
            8.0_f32,
            egui::Color32::from_rgba_unmultiplied(210, 128, 38, 54),
        ),
    );
    painter.circle_stroke(
        geometry.start.lerp(geometry.end, 0.5),
        13.0,
        egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(210, 128, 38)),
    );
}

pub(super) fn paint_canvas_stream_status_badges(
    painter: &egui::Painter,
    geometry: CanvasStreamLineGeometry,
    badges: &[radishflow_studio::StudioGuiCanvasStatusBadgeViewModel],
) {
    if badges.is_empty() {
        return;
    }

    let anchor = geometry.start.lerp(geometry.end, 0.5) + egui::vec2(0.0, -16.0);
    paint_canvas_status_badges(painter, anchor, badges);
}

pub(super) fn paint_canvas_stream_label(
    painter: &egui::Painter,
    canvas_rect: egui::Rect,
    geometry: CanvasStreamLineGeometry,
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
    selected: bool,
    readout: Option<&str>,
) -> Option<egui::Rect> {
    let labeled_readout = readout.and_then(|readout| {
        let label = format!("{} · {readout}", truncate_canvas_label(&stream.name, 18));
        let size = canvas_stream_label_size(&label);
        let rect = stream_label_rect_for_size(geometry, stream, size)?;
        canvas_rect
            .shrink(8.0)
            .contains_rect(rect)
            .then_some((label, rect))
    });
    let (label, label_rect) = match labeled_readout {
        Some(positioned) => positioned,
        None => (
            canvas_stream_label_text(stream),
            canvas_stream_label_rect(geometry, stream)?,
        ),
    };
    let size = canvas_stream_label_size(&label);
    let mut min = label_rect.min;
    min.x = min
        .x
        .clamp(canvas_rect.left() + 8.0, canvas_rect.right() - size.x - 8.0);
    min.y = min
        .y
        .clamp(canvas_rect.top() + 8.0, canvas_rect.bottom() - size.y - 8.0);
    let rect = egui::Rect::from_min_size(min, size);
    let color = topology_color(selected);

    painter.rect_filled(
        rect.translate(egui::vec2(0.0, 1.5)),
        4.0,
        egui::Color32::from_rgba_unmultiplied(30, 42, 54, 24),
    );
    painter.rect_filled(
        rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 238),
    );
    paint_canvas_rect_border(
        painter,
        rect,
        egui::Stroke::new(1.0_f32, color.gamma_multiply(0.7)),
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.5),
        color,
    );
    Some(rect)
}

pub(super) fn canvas_stream_label_rect(
    geometry: CanvasStreamLineGeometry,
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) -> Option<egui::Rect> {
    let label = canvas_stream_label_text(stream);
    stream_label_rect_for_size(geometry, stream, canvas_stream_label_size(&label))
}

fn stream_label_rect_for_size(
    geometry: CanvasStreamLineGeometry,
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
    size: egui::Vec2,
) -> Option<egui::Rect> {
    if stream.sink.is_none() {
        let vertical_offset = canvas_terminal_stream_label_vertical_offset(stream);
        let min = egui::pos2(
            geometry.start.x + 14.0,
            geometry.start.y - size.y * 0.5 + vertical_offset,
        );
        return Some(egui::Rect::from_min_size(min, size));
    }

    if !canvas_connected_stream_has_label_room(geometry, size) {
        return None;
    }

    let center = canvas_stream_label_center(geometry);
    Some(egui::Rect::from_center_size(center, size))
}

fn canvas_connected_stream_has_label_room(
    geometry: CanvasStreamLineGeometry,
    label_size: egui::Vec2,
) -> bool {
    let line_length = (geometry.end - geometry.start).length();
    line_length >= (label_size.x + 24.0).max(86.0)
}

fn canvas_stream_label_center(geometry: CanvasStreamLineGeometry) -> egui::Pos2 {
    let delta = geometry.end - geometry.start;
    let normal = if delta.length() > 1.0 {
        let direction = delta.normalized();
        egui::vec2(-direction.y, direction.x)
    } else {
        egui::vec2(0.0, -1.0)
    };
    geometry.start.lerp(geometry.end, 0.5) + normal * 14.0
}

fn canvas_terminal_stream_label_vertical_offset(
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) -> f32 {
    stream
        .source
        .as_ref()
        .filter(|source| source.port_side_count > 1)
        .map(|source| {
            let middle = (source.port_side_count.saturating_sub(1)) as f32 * 0.5;
            (source.port_side_index as f32 - middle) * 28.0
        })
        .unwrap_or(0.0)
}

fn canvas_stream_label_text(
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) -> String {
    if stream.sink.is_none() {
        return truncate_canvas_label(&stream.name, 18);
    }

    let label = if stream.name == stream.stream_id {
        stream.name.clone()
    } else {
        format!("{} ({})", stream.name, stream.stream_id)
    };
    truncate_canvas_label(&label, 26)
}

fn canvas_stream_label_size(label: &str) -> egui::Vec2 {
    egui::vec2(
        (26.0 + label.chars().count() as f32 * 6.2).clamp(54.0, 188.0),
        20.0,
    )
}

fn paint_canvas_stream_arrow(
    painter: &egui::Painter,
    geometry: CanvasStreamLineGeometry,
    color: egui::Color32,
) {
    let delta = geometry.end - geometry.start;
    let length = delta.length();
    if length <= 1.0 {
        return;
    }

    let direction = delta / length;
    let normal = egui::vec2(-direction.y, direction.x);
    let back = geometry.end - direction * 10.0;
    painter.line_segment(
        [geometry.end, back + normal * 4.5],
        egui::Stroke::new(1.6_f32, color),
    );
    painter.line_segment(
        [geometry.end, back - normal * 4.5],
        egui::Stroke::new(1.6_f32, color),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashed_stream_subdivision_is_limited_to_the_viewport() {
        let clip = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(250.0, 100.0));
        for (y, visible) in [(50.0, true), (120.0, false)] {
            let geometry = CanvasStreamLineGeometry {
                start: egui::pos2(-10_000.0, y),
                end: egui::pos2(10_000.0, y),
            };
            let segment = clip_stream_segment(geometry, clip);
            assert_eq!(segment.is_some(), visible);
            if let Some([start, end]) = segment {
                assert!(start.x.abs() < 0.01);
                assert!((end.x - 250.0).abs() < 0.01);
            }
        }
    }
}
