use super::*;
use radishflow_studio::StudioDocumentLifecycleCommand;

pub(super) struct PendingFormatUpgrade {
    checkpoint: input_departure::ProjectDepartureCheckpoint,
    target_version: u32,
    path: PathBuf,
}

impl ReadyAppState {
    pub(super) fn save_project(&mut self) {
        let snapshot = self.platform_host.snapshot();
        let Some(path) = snapshot.runtime.workspace_document.project_path.as_ref() else {
            self.save_project_as_from_picker();
            return;
        };
        self.request_save_project_as(PathBuf::from(path), false);
    }

    pub(super) fn save_project_as_from_picker(&mut self) {
        if !self.project_dialogs_available() {
            return;
        }
        let Some(path) = self.project_file_picker.pick_save_project_file() else {
            self.cancel_project_save("Save As canceled", "Current workspace remains open.");
            return;
        };
        self.request_save_project_as(path, true);
    }

    pub(super) fn request_save_project_as(&mut self, path: PathBuf, confirm_overwrite: bool) {
        if confirm_overwrite && self.save_as_requires_overwrite_confirmation(&path) {
            self.project_open.pending_save_as_overwrite = Some(path.clone());
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: "Confirm overwrite".into(),
                detail: format!(
                    "Save As target already exists and will be replaced: {}",
                    path.display()
                ),
            });
            return;
        }
        if self.platform_host.project_save_requires_upgrade() {
            self.pending_format_upgrade = Some(PendingFormatUpgrade {
                checkpoint: self.project_departure_checkpoint(),
                target_version: self.platform_host.project_save_version(),
                path,
            });
            self.project_open.pending_save_as_overwrite = None;
            return;
        }
        self.execute_project_save(path, false);
    }

    pub(super) fn confirm_format_upgrade(&mut self) {
        let Some(pending) = self.pending_format_upgrade.take() else {
            return;
        };
        if self.project_departure_checkpoint() != pending.checkpoint
            || self.platform_host.project_save_version() != pending.target_version
        {
            self.cancel_project_save("保存目标已失效", "工程或输入已变化，请重新检查后保存。");
            return;
        }
        self.execute_project_save(pending.path, true);
    }

    fn execute_project_save(&mut self, path: PathBuf, upgrade: bool) {
        let Some(window_id) = self.current_window_id() else {
            return;
        };
        let save_as = self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .project_path
            .as_deref()
            .is_none_or(|current| !paths_match(std::path::Path::new(current), &path));
        let command = if upgrade {
            StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() }
        } else if !save_as {
            StudioDocumentLifecycleCommand::Save
        } else {
            StudioDocumentLifecycleCommand::SaveAs { path: path.clone() }
        };
        match self.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::DocumentLifecycle(command),
        }) {
            Ok(dispatch) => {
                let revision = dispatch.dispatch.window.runtime.workspace_document.revision;
                let pending = self
                    .platform_host
                    .snapshot()
                    .runtime
                    .workspace_document
                    .save_state
                    .pending_input_count;
                self.project_open.path_input = path.display().to_string();
                self.project_open.pending_save_as_overwrite = None;
                let preferences_notice = self.record_and_persist_recent_project(path.clone());
                self.project_open.notice = Some(preferences_notice.unwrap_or(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Info,
                    title: if save_as { "Project saved as" } else { "Project saved" }.into(),
                    detail: format!("Saved revision {revision} to {}. 未提交输入 {pending} 项；草稿未写入文件。", path.display()),
                }));
                self.persist_saved_canvas_layout();
            }
            Err(error) => {
                if save_as {
                    self.project_open.pending_save_as_overwrite = Some(path.clone());
                }
                self.platform_host.record_activity_line(format!(
                    "{} failed: {error}",
                    if save_as { "save as" } else { "save" }
                ));
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: if save_as {
                        "Save As failed"
                    } else {
                        "Project save failed"
                    }
                    .into(),
                    detail: format!(
                        "[{}] {} ({}). Current workspace remains open; choose another target or retry.",
                        error.code().as_str(),
                        error.message(),
                        path.display()
                    ),
                });
            }
        }
    }

    pub(super) fn confirm_pending_save_as_overwrite(&mut self) {
        if let Some(path) = self.project_open.pending_save_as_overwrite.take() {
            self.request_save_project_as(path, false);
        }
    }

    pub(super) fn cancel_pending_save_as_overwrite(&mut self) {
        self.cancel_project_save(
            "Save As canceled",
            "Existing project file was not overwritten.",
        );
    }

    fn cancel_project_save(&mut self, title: &str, detail: &str) {
        self.project_open.pending_save_as_overwrite = None;
        self.pending_format_upgrade = None;
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Info,
            title: title.into(),
            detail: detail.into(),
        });
    }

    fn save_as_requires_overwrite_confirmation(&self, path: &std::path::Path) -> bool {
        path.exists()
            && self
                .platform_host
                .snapshot()
                .runtime
                .workspace_document
                .project_path
                .as_deref()
                .map(std::path::Path::new)
                .is_none_or(|current| !paths_match(current, path))
    }

    pub(super) fn render_format_upgrade(&mut self, ctx: &egui::Context) -> bool {
        let Some(pending) = self.pending_format_upgrade.as_ref() else {
            return false;
        };
        let path = pending.path.display().to_string();
        let version = pending.target_version;
        let zh = self.locale == StudioShellLocale::ZhCn;
        let response = egui::Modal::new(egui::Id::new("project-format-upgrade")).show(ctx, |ui| {
            ui.heading(if zh { "保存为新版工程" } else { "Upgrade project format" });
            ui.label(if zh {
                format!("此工程将保存为 v{version}，保留正式输入和显示设置，旧版程序无法读取。可另存为以保留原件。")
            } else {
                format!("Save as v{version} to preserve formal inputs and display settings. Older versions cannot read it. Use Save As to keep the original.")
            });
            ui.label(path);
            ui.horizontal(|ui| {
                if ui.button(if zh { "升级并保存" } else { "Upgrade and save" }).clicked() {
                    self.confirm_format_upgrade();
                    if !self
                        .platform_host
                        .snapshot()
                        .runtime
                        .workspace_document
                        .save_state
                        .needs_close_confirmation()
                        && self
                            .project_open
                            .pending_close_window_confirmation
                            .is_some()
                        && self.finish_saved_close()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                if ui.button(if zh { "另存为…" } else { "Save As…" }).clicked() {
                    self.pending_format_upgrade = None;
                    self.save_project_as_from_picker();
                }
                if ui.button(if zh { "取消" } else { "Cancel" }).clicked() {
                    self.cancel_project_save("已取消升级", "文件、保存基线和草稿保持不变。");
                }
            });
        });
        if response.should_close() {
            self.cancel_project_save("已取消升级", "文件、保存基线和草稿保持不变。");
        }
        true
    }
}
