use super::*;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::studio_gui_shell) enum EquipmentTaskMode {
    #[default]
    Sizing,
    Rating,
}
impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn create_equipment_task(&mut self, mode: EquipmentTaskMode) {
        self.equipment.creating_task = false;
        self.equipment_command(match mode {
            EquipmentTaskMode::Sizing => EquipmentEditorCommand::CreateTask,
            EquipmentTaskMode::Rating => EquipmentEditorCommand::CreateRatingTask,
        });
    }
    pub(super) fn render_equipment_creation(&mut self, ctx: &egui::Context) {
        if !self.equipment.creating_task {
            return;
        }
        let l = self.locale;
        let response = egui::Modal::new(egui::Id::new("equipment-task-mode")).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 40.0).clamp(220.0, 520.0));
            ui.heading(tr(l, "新建设备任务", "New equipment task"));
            let sizing = ui.radio_value(&mut self.equipment.task_mode, EquipmentTaskMode::Sizing, tr(l, "初步尺寸", "Preliminary sizing"));
            let rating = ui.radio_value(&mut self.equipment.task_mode, EquipmentTaskMode::Rating, tr(l, "已有几何校核", "Given-geometry rating"));
            ui.label(tr(
                l,
                "创建后模式固定。已有几何校核使用给定内径与 LLL → NLL 高度；不预填示例值。",
                "The mode is fixed after creation. Rating uses a given diameter and LLL → NLL height; no example values are filled in.",
            ));
            ui.horizontal_wrapped(|ui| {
                let cancel = ui.button(tr(l, "取消", "Cancel"));
                let create = ui.button(tr(l, "创建任务", "Create task"));
                if !cancel.has_focus() && !create.has_focus() && !create.lost_focus() && !sizing.has_focus() && !rating.has_focus() {
                    cancel.request_focus();
                }
                if cancel.clicked() {
                    self.equipment.creating_task = false;
                }
                if create.clicked() {
                    self.create_equipment_task(self.equipment.task_mode);
                }
            });
        });
        if response.should_close() {
            self.equipment.creating_task = false;
        }
    }
}
