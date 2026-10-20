use super::*;

pub(super) fn render(
    ui: &mut egui::Ui,
    locale: StudioShellLocale,
    inputs: &EngineeringInputs,
    editor: &EquipmentEditorState,
    commands: &mut Vec<EquipmentEditorCommand>,
) {
    ui.strong(tr(locale, "设备任务", "Equipment tasks"));
    for task in &inputs.tasks {
        item(
            ui,
            locale,
            if task.as_sizing().is_some() {
                EquipmentEditorTarget::Task(task.id().clone())
            } else {
                EquipmentEditorTarget::RatingTask(task.id().clone())
            },
            inputs,
            editor,
            commands,
        );
        if let Some(sizing) = task.as_sizing() {
            ui.small(nonempty(
                &sizing.case.name,
                tr(locale, "未命名工况", "Untitled case"),
            ));
        } else {
            ui.small(tr(locale, "已有几何校核", "Given-geometry rating"));
        }
    }
    ui.separator();
    ui.strong(tr(locale, "工程基础", "Engineering basis"));
    for basis in &inputs.basis.basis_entries {
        item(
            ui,
            locale,
            EquipmentEditorTarget::Basis(basis.id.clone()),
            inputs,
            editor,
            commands,
        );
    }
    for target in editor.pending_targets().into_iter().filter(|t|matches!(t,EquipmentEditorTarget::Basis(id) if !inputs.basis.basis_entries.iter().any(|b|b.id==*id))) {item(ui,locale,target,inputs,editor,commands);}
    ui.separator();
    ui.strong(tr(locale, "来源与依据", "Sources and evidence"));
    for source in &inputs.basis.sources {
        item(
            ui,
            locale,
            EquipmentEditorTarget::Source(source.id.clone()),
            inputs,
            editor,
            commands,
        );
    }
}
fn item(
    ui: &mut egui::Ui,
    locale: StudioShellLocale,
    target: EquipmentEditorTarget,
    inputs: &EngineeringInputs,
    editor: &EquipmentEditorState,
    commands: &mut Vec<EquipmentEditorCommand>,
) {
    let pending = editor.draft(&target).is_some_and(|d| d.is_pending());
    let label = format!(
        "{}{}",
        target_label(locale, &target, inputs),
        if pending { " *" } else { "" }
    );
    if ui
        .add(
            egui::Button::new(label)
                .selected(editor.active() == Some(&target))
                .wrap(),
        )
        .clicked()
    {
        commands.push(EquipmentEditorCommand::Select(target));
    }
}

impl ReadyAppState {
    pub(in super::super) fn command_palette_items(
        &self,
        commands: &radishflow_studio::StudioGuiWindowCommandAreaModel,
        query: &str,
    ) -> Vec<radishflow_studio::StudioGuiWindowCommandPaletteItemModel> {
        let mut items = commands.palette_items(query);
        if self.screen != StudioShellScreen::Equipment {
            return items;
        }
        let session = &self.platform_host.modeling_inputs().workspace.engineering;
        for (id, enabled, label) in [
            (
                radishflow_studio::EDIT_UNDO_COMMAND_ID,
                session.can_undo(),
                tr(self.locale, "撤销设备修改", "Undo equipment change"),
            ),
            (
                radishflow_studio::EDIT_REDO_COMMAND_ID,
                session.can_redo(),
                tr(self.locale, "重做设备修改", "Redo equipment change"),
            ),
        ] {
            items.retain(|item| item.command_id != id);
            if let Some(mut item) = commands
                .palette_items("")
                .into_iter()
                .find(|item| item.command_id == id)
            {
                item.enabled = enabled;
                item.label = label.into();
                item.detail=tr(self.locale,"设备、来源与基础条目的正式历史；有草稿时先确认。","Formal equipment, source and basis history; pending inputs require confirmation.").into();
                item.hover_text = item.detail.clone();
                item.menu_path_text = tr(self.locale, "设备 / 编辑", "Equipment / Edit").into();
                let searchable =
                    format!("{} {} {}", item.command_id, item.label, item.detail).to_lowercase();
                if query
                    .to_lowercase()
                    .split_whitespace()
                    .all(|word| searchable.contains(word))
                {
                    items.push(item);
                }
            }
        }
        items
    }
}
