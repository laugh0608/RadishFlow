use super::{FlowPlanConfiguration, PortDirection, TearSpecification, egui, recycle_review, tr};
use rf_types::RfErrorContext;
use rf_ui::WorkspaceState;

pub(super) fn render_applied_plan(ui: &mut egui::Ui, workspace: &WorkspaceState, zh: bool) {
    let applied = workspace.flow_plan.current();
    let flow = &workspace.document.flowsheet;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong(tr(zh, "已应用计划", "Applied plan"));
            ui.small(format!(
                "{} {}",
                tr(zh, "计划修订", "Plan revision"),
                workspace.flow_plan.revision()
            ));
        });
        let Err(error) = applied.validate_for(flow) else {
            ui.strong(tr(zh, "拓扑与绑定已就绪", "Topology and bindings ready"));
            ui.label(tr(
                zh,
                "运行时继续检查输入与物性。",
                "Run also checks physical inputs and the property package.",
            ));
            return;
        };
        ui.colored_label(
            ui.visuals().error_fg_color,
            tr(zh, "拓扑与绑定未就绪", "Topology and bindings not ready"),
        );
        let context = error.context();
        let Some(tear) = outside_tear(applied, context) else {
            ui.label(error.to_string());
            return;
        };
        // A terminal stream can have the same diagnostic code and a configured but absent sink.
        let confirmed = |end, direction| {
            context.related_port_targets().contains(end)
                && recycle_review::material_port_is_bound(flow, &tear.stream_id, end, direction)
        };
        let source = confirmed(&tear.source, PortDirection::Outlet);
        let sink = confirmed(&tear.sink, PortDirection::Inlet);
        if source && sink {
            ui.label(tr(
                zh,
                "撕裂流股位于循环块外。",
                "The tear stream is outside a cyclic block.",
            ));
        } else {
            ui.label(error.to_string());
        }
        ui.separator();
        let targets = |ui: &mut egui::Ui| {
            ui.label(format!("{}  {}", tr(zh, "流股", "Stream"), tear.stream_id));
            for (visible, title, port) in [
                (source, tr(zh, "源端口", "Source port"), &tear.source),
                (sink, tr(zh, "汇端口", "Sink port"), &tear.sink),
            ] {
                if visible {
                    ui.label(format!("{title}  {}.{}", port.unit_id, port.port_name));
                }
            }
        };
        if ui.available_width() < 600.0 {
            ui.vertical(targets);
        } else {
            ui.horizontal_wrapped(targets);
        }
    });
}

fn outside_tear<'a>(
    applied: &'a FlowPlanConfiguration,
    context: &RfErrorContext,
) -> Option<&'a TearSpecification> {
    if context.diagnostic_code() != Some("solver.partitioned.outside_tear") {
        return None;
    }
    let FlowPlanConfiguration::Partitioned(options) = applied else {
        return None;
    };
    let [stream] = context.related_stream_ids() else {
        return None;
    };
    options.tears.iter().find(|tear| &tear.stream_id == stream)
}
