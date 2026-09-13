use super::*;

impl ReadyAppState {
    pub(super) fn persist_saved_canvas_layout(&mut self) {
        let window = self.platform_host.snapshot().window_model();
        let Some(path) = window.runtime.workspace_document.project_path.as_deref() else {
            return;
        };
        let positions = window
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .filter_map(|unit| {
                unit.layout_position
                    .map(|position| (rf_types::UnitId::new(&unit.unit_id), position))
            })
            .collect();
        if let Err(error) = radishflow_studio::save_persisted_canvas_unit_positions(
            std::path::Path::new(path),
            &positions,
        ) {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: "项目已保存，布局保存失败".to_string(),
                detail: format!(
                    "[{}] {}。工程数据已保存，可重试保存布局。",
                    error.code().as_str(),
                    error.message()
                ),
            });
        }
    }
}
