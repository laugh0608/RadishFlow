use super::*;
use rf_ui::{
    WorkspaceState, project_query::ProjectObjectId, variable_browser::VariableReferenceError,
};

pub(super) fn selected_variable(selection: &BrowserSelection) -> Option<VariableId> {
    let ProjectObjectId::Flow(object) = &selection.object.object else {
        return None;
    };
    let ProjectField::Flow { section, field } = selection.field.as_ref()? else {
        return None;
    };
    Some(VariableId {
        document: selection.object.document.clone(),
        object: object.clone(),
        section: *section,
        field: field.clone(),
    })
}

pub(super) fn variable_selection(variable: VariableId) -> BrowserSelection {
    BrowserSelection {
        object: ProjectObjectRef {
            document: variable.document,
            object: ProjectObjectId::Flow(variable.object),
        },
        field: Some(ProjectField::Flow {
            section: variable.section,
            field: variable.field,
        }),
    }
}

impl ReadyAppState {
    pub(super) fn copy_stable_reference(
        &self,
        selection: &BrowserSelection,
    ) -> Result<String, String> {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if !self.variable_browser.belongs_to(workspace) || self.read_only_project.is_some() {
            return Err(tr(
                self.locale,
                "工程工作区不可用。",
                "Project workspace unavailable.",
            )
            .into());
        }
        let variable = selected_variable(selection).ok_or_else(|| {
            error_label(VariableReferenceError::UnsupportedField, self.locale).to_owned()
        })?;
        StableVariableReference::for_saved_variable(workspace, variable)
            .map(|reference| reference.to_string())
            .map_err(|error| error_label(error, self.locale).into())
    }

    pub(super) fn open_stable_reference(&mut self, text: &str) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        match text
            .parse::<StableVariableReference>()
            .and_then(|reference| {
                reference.resolve(workspace)?;
                Ok(reference)
            }) {
            Ok(reference) => {
                self.request_browser_navigation(BrowserNavigation::StableReference(reference))
            }
            Err(error) => {
                self.variable_browser.notice = Some(error_label(error, self.locale).into())
            }
        }
    }
}

pub(super) fn error_label(
    error: VariableReferenceError,
    locale: StudioShellLocale,
) -> &'static str {
    use VariableReferenceError::*;
    let (zh, en) = match error {
        Malformed => ("变量引用格式无效。", "Invalid variable reference format."),
        UnsupportedVersion => (
            "此变量引用版本不受支持。",
            "Unsupported variable reference version.",
        ),
        UnsupportedField => (
            "此字段类型尚不支持稳定引用。",
            "This field type has no stable reference support.",
        ),
        DifferentDocument => (
            "引用属于其他工程。",
            "This reference belongs to a different project.",
        ),
        IdentitiesUnavailable => (
            "当前工程尚未启用稳定引用。",
            "Stable references are not enabled for this project.",
        ),
        ObjectMissing => (
            "引用的对象已不存在。",
            "The referenced object no longer exists.",
        ),
        ObjectNotSaved => (
            "保存此对象后可复制引用。",
            "Save this object before copying its reference.",
        ),
        ObjectRecreated => (
            "对象 ID 已被重用，引用指向原对象。",
            "The object ID was reused. This reference points to the original object.",
        ),
        FieldMissing => (
            "对象上已无此字段。",
            "The object no longer exposes this field.",
        ),
    };
    tr(locale, zh, en)
}

pub(super) fn render_reference_controls(
    ui: &mut egui::Ui,
    state: &mut VariableBrowserState,
    workspace: &WorkspaceState,
    locale: StudioShellLocale,
    intent: &mut Option<BrowserIntent>,
) {
    let input_id = ui.id().with("reference-input");
    ui.horizontal_wrapped(|ui| {
        if ui
            .small_button(tr(locale, "粘贴变量引用…", "Paste variable reference…"))
            .clicked()
        {
            state.reference_input = Some(String::new());
            ui.memory_mut(|memory| memory.request_focus(input_id));
        }
        if !workspace.object_identities_enabled()
            && ui
                .small_button(tr(locale, "启用稳定引用…", "Enable stable references…"))
                .clicked()
        {
            *intent = Some(BrowserIntent::InitializeReferences);
        }
    });
    if workspace.object_identities_enabled() || state.reference_input.is_some() {
        let location = workspace.document_path.as_ref().map_or_else(
            || tr(locale, "尚未保存", "Unsaved").into(),
            |path| path.display().to_string(),
        );
        ui.add(egui::Label::new(location).wrap());
    }
    if let Some(input) = &mut state.reference_input {
        let edit = ui.add(
            egui::TextEdit::singleline(input)
                .id(input_id)
                .hint_text("rfvar:1/…")
                .desired_width(f32::INFINITY),
        );
        let enter = edit.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
        let mut cancel = false;
        ui.horizontal_wrapped(|ui| {
            if enter || ui.button(tr(locale, "跳转", "Go to reference")).clicked() {
                *intent = Some(BrowserIntent::OpenReference(input.clone()));
            }
            cancel = ui.button(tr(locale, "取消", "Cancel")).clicked();
        });
        if cancel {
            state.reference_input = None;
        }
    }
}

pub(super) fn render_copy_reference(
    ui: &mut egui::Ui,
    workspace: &WorkspaceState,
    selection: &BrowserSelection,
    locale: StudioShellLocale,
    intent: &mut Option<BrowserIntent>,
) {
    let Some(variable) = selected_variable(selection).filter(StableVariableReference::supports)
    else {
        return;
    };
    if !workspace.object_identities_enabled() {
        return;
    }
    let enabled = workspace
        .saved_object_creation_id(&variable.object)
        .is_some();
    if ui
        .add_enabled(
            enabled,
            egui::Button::new(tr(locale, "复制稳定引用", "Copy stable reference")),
        )
        .on_disabled_hover_text(error_label(VariableReferenceError::ObjectNotSaved, locale))
        .clicked()
    {
        *intent = Some(BrowserIntent::CopyReference(selection.clone()));
    }
}
