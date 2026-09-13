use super::*;

impl ReadyAppState {
    pub(super) fn record_canvas_viewport_navigation_for_command(
        &mut self,
        command_id: &str,
        canvas_navigation: Option<&radishflow_studio::StudioGuiCanvasCommandTargetViewModel>,
    ) -> bool {
        if canvas_navigation.is_none() {
            return false;
        }
        let snapshot = self.platform_host.snapshot();
        let window = snapshot.window_model();
        let focus = window.canvas.widget.view().viewport.focus.as_ref();
        if let Some(anchor_label) = self
            .canvas_viewport_navigation
            .request_for_command(command_id, focus)
        {
            self.last_area_focus = Some(StudioGuiWindowAreaId::Canvas);
            if let Some(target) = canvas_navigation {
                let result = radishflow_studio::StudioGuiCanvasCommandResultViewModel::located(
                    target.clone(),
                    anchor_label,
                );
                self.platform_host
                    .record_activity_line(result.activity_line.clone());
                self.canvas_command_result = Some(result);
            }
            return true;
        }
        false
    }

    pub(super) fn reconcile_canvas_viewport_navigation(
        &mut self,
        view: &radishflow_studio::StudioGuiCanvasViewModel,
    ) {
        let Some(result) = self.canvas_command_result.as_ref() else {
            return;
        };
        let Some(previous_anchor) = result.anchor_label.as_ref() else {
            return;
        };
        let target = &result.target;
        let current = view.object_list.items.iter().find(|item| {
            item.kind_label == target.kind_label && item.target_id == target.target_id
        });
        let Some(current) = current else {
            let expired_anchor = previous_anchor.clone();
            self.canvas_viewport_navigation.active_anchor = None;
            self.canvas_command_result = Some(
                radishflow_studio::StudioGuiCanvasCommandResultViewModel::anchor_expired(
                    target.clone(),
                    expired_anchor,
                ),
            );
            return;
        };
        let selected = view.viewport.focus.as_ref().is_some_and(|focus| {
            focus.kind_label == target.kind_label && focus.target_id == target.target_id
        });
        if !selected {
            // A selection change is not evidence that the previous object disappeared.
            self.canvas_viewport_navigation.active_anchor = None;
            self.canvas_command_result = None;
            return;
        }
        if previous_anchor != &current.viewport_anchor_label
            || self.canvas_viewport_navigation.active_anchor.is_none()
        {
            let target = current.command_target();
            let anchor = self
                .canvas_viewport_navigation
                .request_anchor(&current.viewport_anchor_label);
            self.canvas_command_result = Some(
                radishflow_studio::StudioGuiCanvasCommandResultViewModel::located(target, anchor),
            );
        }
    }

    pub(super) fn canvas_object_navigation_request(
        &self,
        command_id: &str,
    ) -> Option<radishflow_studio::StudioGuiCanvasCommandTargetViewModel> {
        let snapshot = self.platform_host.snapshot();
        let window = snapshot.window_model();
        if let Some(item) = window
            .canvas
            .widget
            .view()
            .object_list
            .items
            .iter()
            .find(|item| item.command_id == command_id)
        {
            return Some(item.command_target());
        }

        radishflow_studio::inspector_target_from_command_id(command_id).map(|target| {
            let (kind_label, target_id) = match target {
                rf_ui::InspectorTarget::Unit(unit_id) => ("Unit", unit_id.as_str().to_string()),
                rf_ui::InspectorTarget::Stream(stream_id) => {
                    ("Stream", stream_id.as_str().to_string())
                }
            };
            radishflow_studio::StudioGuiCanvasCommandTargetViewModel {
                kind_label,
                label: target_id.clone(),
                target_id,
                viewport_anchor_label: None,
                command_id: command_id.to_string(),
            }
        })
    }

    pub(super) fn record_canvas_object_navigation_feedback(
        &mut self,
        request: Option<&radishflow_studio::StudioGuiCanvasCommandTargetViewModel>,
        viewport_requested: bool,
        error_message: Option<&str>,
    ) {
        let Some(request) = request else {
            return;
        };
        if viewport_requested {
            return;
        }

        let result = match error_message {
            Some(error_message) => {
                radishflow_studio::StudioGuiCanvasCommandResultViewModel::dispatch_failed(
                    request.clone(),
                    error_message,
                )
            }
            None => radishflow_studio::StudioGuiCanvasCommandResultViewModel::anchor_unavailable(
                request.clone(),
            ),
        };
        self.platform_host
            .record_activity_line(result.activity_line.clone());
        self.canvas_command_result = Some(result);
    }
}
