use super::*;
use crate::studio_gui_shell::tests::reconnect_picker::{
    click, frame, mixer_with_two_open_ports, node, paint_frame,
};

#[derive(Default)]
struct PaintedPorts {
    fixed: Option<egui::Pos2>,
    candidates: Vec<egui::Pos2>,
    chosen: Option<egui::Pos2>,
    highlighted: Option<egui::Pos2>,
    dashes: Vec<[egui::Pos2; 2]>,
}

impl PaintedPorts {
    fn from_output(output: &egui::FullOutput) -> Self {
        let mut ports = Self::default();
        for shape in &output.shapes {
            match &shape.shape {
                egui::Shape::Path(path)
                    if path.stroke.color == egui::epaint::ColorMode::Solid(FIXED) =>
                {
                    ports.fixed = Some(egui::Rect::from_points(&path.points).center());
                }
                egui::Shape::Circle(circle) if circle.stroke.color == PREVIEW => {
                    ports.candidates.push(circle.center);
                    if circle.fill == PREVIEW {
                        assert!(ports.chosen.replace(circle.center).is_none());
                    }
                }
                egui::Shape::Rect(rect) if rect.stroke.color == PREVIEW => {
                    ports.highlighted = Some(rect.rect.center());
                }
                egui::Shape::LineSegment { points, stroke }
                    if stroke.color == PREVIEW && stroke.width == 2.0 =>
                {
                    ports.dashes.push(*points);
                }
                _ => {}
            }
        }
        ports
    }
}

#[test]
fn reconnect_canvas_tracks_each_picker_row_and_the_committed_material_line_after_panning() {
    for (side, entry, first, second) in [
        (
            "sink",
            "Choose sink port…",
            "Mixer · mixer-1:inlet_a",
            "Mixer · mixer-1:inlet_b",
        ),
        (
            "source",
            "Choose source port…",
            "Feed A · feed-a:outlet",
            "Feed B · feed-b:outlet",
        ),
    ] {
        let mut app = mixer_with_two_open_ports(side);
        let before = app.platform_host.modeling_inputs().clone();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let tree = frame(&mut app, &ctx, None);
        frame(&mut app, &ctx, Some(click(node(&tree, entry))));
        let opened = paint_frame(&mut app, &ctx, None);
        let marks = PaintedPorts::from_output(&opened);
        assert_eq!(marks.candidates.len(), 2);
        let fixed = marks.fixed.unwrap();
        let first_pos = marks.highlighted.unwrap();
        assert!(marks.chosen.is_none() && marks.dashes.is_empty());

        let tree = opened.platform_output.accesskit_update.unwrap();
        let second_id = node(&tree, second);
        let bounds = tree
            .nodes
            .iter()
            .find(|(id, _)| *id == second_id)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        let hover = egui::pos2(
            (bounds.x0 + bounds.x1) as f32 / 2.0,
            (bounds.y0 + bounds.y1) as f32 / 2.0,
        );
        let hovered = paint_frame(&mut app, &ctx, Some(egui::Event::PointerMoved(hover)));
        let marks = PaintedPorts::from_output(&hovered);
        let second_pos = marks.highlighted.unwrap();
        assert!(marks.candidates.contains(&second_pos) && first_pos != second_pos);
        assert!(marks.chosen.is_none() && marks.dashes.is_empty());

        frame(&mut app, &ctx, Some(click(node(&tree, first))));
        let first_choice = paint_frame(&mut app, &ctx, None);
        assert_eq!(
            PaintedPorts::from_output(&first_choice).chosen,
            Some(first_pos)
        );
        frame(&mut app, &ctx, Some(click(second_id)));
        let selected = paint_frame(&mut app, &ctx, None);
        assert_eq!(
            PaintedPorts::from_output(&selected).chosen,
            Some(second_pos)
        );

        let pan = egui::vec2(-18.0, 9.0);
        app.canvas_initial_viewport_fit
            .set_offset(app.canvas_initial_viewport_fit.offset + pan);
        let panned = paint_frame(&mut app, &ctx, None);
        let marks = PaintedPorts::from_output(&panned);
        assert_eq!(marks.fixed, Some(fixed + pan));
        assert_eq!(marks.chosen, Some(second_pos + pan));
        let (source, sink) = if side == "sink" {
            (fixed + pan, second_pos + pan)
        } else {
            (second_pos + pan, fixed + pan)
        };
        assert_eq!(marks.dashes.first().unwrap()[0], source);
        assert!(marks.dashes.last().unwrap()[1].distance(sink) <= 4.1);
        assert_eq!(app.platform_host.modeling_inputs(), &before);
        let confirm = node(
            panned.platform_output.accesskit_update.as_ref().unwrap(),
            "Confirm reconnection",
        );
        frame(&mut app, &ctx, Some(click(confirm)));
        let committed = paint_frame(&mut app, &ctx, None);
        assert!(PaintedPorts::from_output(&committed).fixed.is_none());
        assert!(committed.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::LineSegment {points, ..} if *points == [source, sink])));
    }
}

#[test]
fn preview_detours_around_an_intervening_unit_without_changing_either_port() {
    let a = egui::pos2(90.0, 60.0);
    let b = egui::pos2(320.0, 68.0);
    let middle = egui::Rect::from_min_max(egui::pos2(112.0, 20.0), egui::pos2(290.0, 102.0));
    let path = preview_path(a, b, std::iter::once(middle));
    assert_eq!(path.first(), Some(&a));
    assert_eq!(path.last(), Some(&b));
    for segment in path.windows(2) {
        assert!(
            streams::clip_stream_segment(
                CanvasStreamLineGeometry {
                    start: segment[0],
                    end: segment[1]
                },
                middle
            )
            .is_none()
        );
    }
    assert_eq!(preview_path(a, b, std::iter::empty()), [a, b]);
}

#[test]
fn reconnect_drops_painted_preview_on_cancel_or_input_change() {
    let mut app = mixer_with_two_open_ports("sink");
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let before = app.platform_host.modeling_inputs().clone();
    for cancel in [true, false] {
        let tree = frame(&mut app, &ctx, None);
        frame(
            &mut app,
            &ctx,
            Some(click(node(&tree, "Choose sink port…"))),
        );
        let tree = frame(&mut app, &ctx, None);
        frame(
            &mut app,
            &ctx,
            Some(click(node(&tree, "Mixer · mixer-1:inlet_b"))),
        );
        let selected = paint_frame(&mut app, &ctx, None);
        assert!(!PaintedPorts::from_output(&selected).dashes.is_empty());
        if cancel {
            let tree = selected.platform_output.accesskit_update.unwrap();
            frame(&mut app, &ctx, Some(click(node(&tree, "Cancel"))));
            assert_eq!(app.platform_host.modeling_inputs(), &before);
        } else {
            // Undoing the second disconnect reoccupies inlet_b.
            app.dispatch_ui_command("edit.undo");
        }
        let cleared = PaintedPorts::from_output(&paint_frame(&mut app, &ctx, None));
        assert!(
            cleared.fixed.is_none() && cleared.candidates.is_empty() && cleared.dashes.is_empty()
        );
    }
}
