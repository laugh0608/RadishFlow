use super::*;
use radishflow_studio::{ProjectSaveCompatibility, StudioDocumentLifecycleCommand};

pub(super) struct PendingFormatUpgrade {
    checkpoint: input_departure::ProjectDepartureCheckpoint,
    compatibility: ProjectSaveCompatibility,
    target_contents: Option<Vec<u8>>,
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
        let compatibility = match self.platform_host.project_save_compatibility(&path) {
            Ok(value) => value,
            Err(error) => {
                self.record_project_save_failure(path, &error);
                return;
            }
        };
        if let Some(compatibility) = compatibility {
            let target_contents = match read_save_target(&path) {
                Ok(value) => value,
                Err(error) => {
                    self.record_project_save_failure(
                        path,
                        &rf_types::RfError::invalid_input(format!("inspect save target: {error}")),
                    );
                    return;
                }
            };
            self.pending_format_upgrade = Some(PendingFormatUpgrade {
                checkpoint: self.project_departure_checkpoint(),
                compatibility,
                target_contents,
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
            || self
                .platform_host
                .project_save_compatibility(&pending.path)
                .ok()
                != Some(Some(pending.compatibility))
            || read_save_target(&pending.path).ok().as_ref() != Some(&pending.target_contents)
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
            Err(error) => self.record_project_save_failure(path, &error),
        }
    }

    fn record_project_save_failure(&mut self, path: PathBuf, error: &rf_types::RfError) {
        let save_as = self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .project_path
            .as_deref()
            .is_none_or(|current| !paths_match(std::path::Path::new(current), &path));
        self.pending_format_upgrade = None;
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
        let compatibility = pending.compatibility;
        let zh = self.locale == StudioShellLocale::ZhCn;
        let response = egui::Modal::new(egui::Id::new("project-format-upgrade")).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 40.0).clamp(220.0, 520.0));
            let capability_only = compatibility == ProjectSaveCompatibility::RatingCapability;
            ui.heading(match (zh, capability_only) {
                (true, true) => "确认校核任务兼容性", (false, true) => "Confirm rating compatibility",
                (true, false) => "保存为新版工程", (false, false) => "Upgrade project format",
            });
            match compatibility {
                ProjectSaveCompatibility::RatingCapability => {
                    ui.label(if zh { "工程仍为 v3，新增校核任务后，仅支持初步尺寸的旧版本将只能只读检查。可另存为以保留原件。" }
                    else { "The project remains v3. Older versions that only support sizing will open the rating project read-only. Use Save As to keep the original." });
                }
                ProjectSaveCompatibility::Format { version, includes_rating } => {
                    ui.label(if zh {
                        format!("此工程将保存为 v{version}，保留正式输入和显示设置，旧版程序无法读取。可另存为以保留原件。")
                    } else {
                        format!("Save as v{version} to preserve formal inputs and display settings. Older versions cannot read it. Use Save As to keep the original.")
                    });
                    if includes_rating {
                        ui.label(if zh { "含校核任务；仅支持初步尺寸的 v3 旧版本只能只读检查。" }
                        else { "Contains rating tasks; v3 versions that only support sizing can inspect it read-only." });
                    }
                }
            }
            ui.label(path);
            ui.horizontal_wrapped(|ui| {
                let cancel = ui.button(if zh { "取消" } else { "Cancel" });
                let save_as = ui.button(if zh { "另存为…" } else { "Save As…" });
                let confirm = ui.button(match (zh, capability_only) {
                    (true, true) => "继续保存", (false, true) => "Continue and save",
                    (true, false) => "升级并保存", (false, false) => "Upgrade and save",
                });
                if !cancel.has_focus() && !save_as.has_focus() && !confirm.has_focus()
                    && !save_as.lost_focus() && !confirm.lost_focus() {
                    cancel.request_focus();
                }
                if confirm.clicked() {
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
                if save_as.clicked() {
                    self.pending_format_upgrade = None;
                    self.save_project_as_from_picker();
                }
                if cancel.clicked() {
                    self.cancel_project_save("已取消升级", "文件、保存基线和草稿保持不变。");
                }
            });
        });
        if response.should_close() && self.pending_format_upgrade.is_some() {
            self.cancel_project_save("已取消升级", "文件、保存基线和草稿保持不变。");
        }
        true
    }
}

fn read_save_target(path: &std::path::Path) -> std::io::Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
