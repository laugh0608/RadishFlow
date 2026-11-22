use super::locale::tr;
use super::*;
use radishflow_studio::{
    PreparedObjectIdentityInitialization, StudioDocumentLifecycleCommand,
    prepare_object_identity_initialization,
};

pub(super) enum PendingIdentityInitialization {
    ChooseDestination,
    Confirm(Box<PreparedObjectIdentityInitialization>),
}

impl ReadyAppState {
    pub(super) fn request_object_identity_initialization(&mut self) {
        if self.read_only_project.is_none()
            && !self
                .platform_host
                .modeling_inputs()
                .workspace
                .object_identities_enabled()
        {
            self.pending_identity_initialization =
                Some(PendingIdentityInitialization::ChooseDestination);
            self.command_palette.close();
        }
    }

    pub(super) fn prepare_identity_destination(&mut self, path: PathBuf) {
        match prepare_object_identity_initialization(self.platform_host.modeling_inputs(), &path) {
            Ok(prepared) => {
                self.pending_identity_initialization =
                    Some(PendingIdentityInitialization::Confirm(Box::new(prepared)))
            }
            Err(error) => {
                self.pending_identity_initialization = None;
                self.identity_save_notice(ProjectOpenNoticeLevel::Error, error.to_string());
            }
        }
    }

    pub(super) fn confirm_identity_initialization(&mut self) {
        let Some(PendingIdentityInitialization::Confirm(prepared)) =
            self.pending_identity_initialization.take()
        else {
            return;
        };
        let Some(window_id) = self.current_window_id() else {
            return;
        };
        let path = prepared.path().to_owned();
        match self.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::DocumentLifecycle(
                StudioDocumentLifecycleCommand::InitializeObjectIdentities(prepared),
            ),
        }) {
            Ok(_) => {
                self.project_open.path_input = path.display().to_string();
                let notice = self.record_and_persist_recent_project(path);
                self.identity_save_notice(
                    ProjectOpenNoticeLevel::Info,
                    tr(
                        self.locale,
                        "稳定引用已启用。",
                        "Stable references are enabled.",
                    )
                    .into(),
                );
                if notice.is_some() {
                    self.project_open.notice = notice;
                }
                self.persist_saved_canvas_layout();
            }
            Err(error) => {
                self.identity_save_notice(ProjectOpenNoticeLevel::Error, error.to_string())
            }
        }
    }

    fn identity_save_notice(&mut self, level: ProjectOpenNoticeLevel, detail: String) {
        self.variable_browser.set_notice(detail.clone());
        self.project_open.notice = Some(ProjectOpenNotice {
            level,
            title: tr(self.locale, "启用稳定引用", "Enable stable references").into(),
            detail,
        });
    }

    pub(super) fn render_identity_initialization(&mut self, ctx: &egui::Context) -> bool {
        use PendingIdentityInitialization::*;
        if self.pending_identity_initialization.is_none() {
            return false;
        }
        let current_path = self
            .platform_host
            .modeling_inputs()
            .workspace
            .document_path
            .clone();
        let dialogs = self.project_file_picker.supports_project_dialogs();
        let pending = self
            .pending_identity_initialization
            .as_ref()
            .expect("checked above");
        let locale = self.locale;
        let mut choose_copy = false;
        let mut choose_in_place = false;
        let mut confirm = false;
        let mut cancel = false;
        let response = egui::Modal::new(egui::Id::new("initialize-object-identities")).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 40.0).clamp(220.0, 520.0));
            ui.heading(tr(locale, "启用稳定引用", "Enable stable references"));
            match pending {
                ChooseDestination => {
                    ui.label(tr(locale,
                        "选择保存位置。建议保存新副本，以保留旧版可编辑的原工程。",
                        "Choose where to save. Save a new copy to keep the original editable in older versions."));
                    ui.horizontal_wrapped(|ui| {
                        choose_copy = ui.add_enabled(dialogs, egui::Button::new(tr(locale, "保存新副本…", "Save new copy…"))).clicked();
                        choose_in_place = ui.add_enabled(current_path.is_some(), egui::Button::new(tr(locale, "升级原文件", "Upgrade original file"))).clicked();
                        cancel = ui.button(tr(locale, "取消", "Cancel")).clicked();
                    });
                }
                Confirm(prepared) => {
                    ui.add(egui::Label::new(prepared.path().display().to_string()).wrap());
                    if prepared.replaces_file() {
                        ui.strong(tr(locale, "确认替换这个现有文件。", "Confirm replacement of this existing file."));
                    }
                    ui.label(match prepared.source_version() {
                        Some(4) => tr(locale, "在 v4 工程中增加对象身份能力。", "Add object identity support to the v4 project."),
                        Some(_) => tr(locale, "将工程升级为 v4 并保存对象身份。", "Upgrade the project to v4 and save object identities."),
                        None => tr(locale, "保存为带对象身份的 v4 工程。", "Save as a v4 project with object identities."),
                    });
                    ui.label(tr(locale,
                        "早于 v4 的 Studio 和 CLI v1–v3 无法打开此工程。旧 v4 Studio 只能只读检查，CLI v4–v7 只能查询安全元数据。",
                        "Studio versions before v4 and CLI v1–v3 reject this file. Older v4 Studio versions can inspect it read-only, and CLI v4–v7 can query safe metadata only."));
                    ui.label(tr(locale, "已提交的修改将一并保存，未提交草稿留在编辑器。", "Committed changes will be saved. Pending drafts remain in the editor."));
                    ui.horizontal_wrapped(|ui| {
                        cancel = ui.button(tr(locale, "取消", "Cancel")).clicked();
                        confirm = ui.button(tr(locale, "启用并保存", "Enable and save")).clicked();
                    });
                }
            }
        });
        if cancel || response.should_close() {
            self.pending_identity_initialization = None;
        } else if choose_copy {
            if let Some(path) = self.project_file_picker.pick_save_project_file() {
                self.prepare_identity_destination(path);
            }
        } else if choose_in_place {
            if let Some(path) = current_path {
                self.prepare_identity_destination(path);
            }
        } else if confirm {
            self.confirm_identity_initialization();
        }
        true
    }
}
