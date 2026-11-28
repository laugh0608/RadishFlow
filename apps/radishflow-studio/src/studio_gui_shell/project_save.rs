use super::file_target::save_target_fingerprint;
use super::locale::tr;
use super::*;
use radishflow_studio::{
    PreparedFlowPlanSave, ProjectSaveCompatibility, StudioDocumentLifecycleCommand,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SaveTargetReview {
    workspace: input_departure::ProjectDepartureCheckpoint,
    target_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingSaveAs {
    pub(super) path: PathBuf,
    action: SaveAsAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SaveAsAction {
    Overwrite {
        review: Box<SaveTargetReview>,
        compatibility: Option<ProjectSaveCompatibility>,
    },
    Retry,
}

impl PendingSaveAs {
    pub(super) fn retry(path: PathBuf) -> Self {
        Self {
            path,
            action: SaveAsAction::Retry,
        }
    }

    pub(super) fn button_label(&self) -> ShellText {
        match self.action {
            SaveAsAction::Overwrite { .. } => ShellText::ConfirmSaveAsOverwrite,
            SaveAsAction::Retry => ShellText::RetrySaveAs,
        }
    }
}

pub(super) enum PendingFormatUpgrade {
    Legacy {
        review: Box<SaveTargetReview>,
        compatibility: ProjectSaveCompatibility,
        path: PathBuf,
    },
    FlowPlan {
        prepared: Box<PreparedFlowPlanSave>,
        overwrite: bool,
    },
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
        self.project_open.pending_save_as_overwrite = None;
        self.pending_format_upgrade = None;
        if self.read_only_project.is_some() {
            return;
        }
        let prepared = match self.platform_host.prepare_flow_plan_save(&path) {
            Ok(value) => value.map(Box::new),
            Err(error) => {
                self.record_project_save_failure(path, &error);
                self.project_open.pending_save_as_overwrite = None;
                return;
            }
        };
        if let Some(prepared) = prepared {
            let overwrite = prepared.replaces_file() && !self.is_current_project_path(&path);
            if overwrite || prepared.compatibility().is_some() {
                self.pending_format_upgrade = Some(PendingFormatUpgrade::FlowPlan {
                    prepared,
                    overwrite,
                });
            } else {
                self.dispatch_project_save(
                    path,
                    StudioDocumentLifecycleCommand::SavePreparedFlowPlan {
                        prepared,
                        capability_confirmed: false,
                    },
                );
            }
            return;
        }
        let compatibility = match self.platform_host.project_save_compatibility(&path) {
            Ok(value) => value,
            Err(error) => {
                self.record_project_save_failure(path, &error);
                return;
            }
        };
        let check_overwrite = confirm_overwrite && !self.is_current_project_path(&path);
        if check_overwrite || compatibility.is_some() {
            let target_sha256 = match save_target_fingerprint(&path) {
                Ok(value) => value,
                Err(error) => {
                    self.record_project_save_failure(path, &error);
                    return;
                }
            };
            let overwrite = check_overwrite && target_sha256.is_some();
            let capture_review = || {
                Box::new(SaveTargetReview {
                    workspace: self.project_departure_checkpoint(),
                    target_sha256,
                })
            };
            if overwrite {
                let review = capture_review();
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Warning,
                    title: "Confirm overwrite".into(),
                    detail: format!(
                        "Save As target already exists and will be replaced: {}",
                        path.display()
                    ),
                });
                self.project_open.pending_save_as_overwrite = Some(PendingSaveAs {
                    path,
                    action: SaveAsAction::Overwrite {
                        review,
                        compatibility,
                    },
                });
                return;
            }
            if let Some(compatibility) = compatibility {
                self.pending_format_upgrade = Some(PendingFormatUpgrade::Legacy {
                    path,
                    review: capture_review(),
                    compatibility,
                });
                return;
            }
        }
        self.execute_project_save(path, false);
    }

    pub(super) fn confirm_format_upgrade(&mut self) -> bool {
        let Some(pending) = self.pending_format_upgrade.take() else {
            return false;
        };
        let PendingFormatUpgrade::Legacy {
            review,
            compatibility,
            path,
        } = pending
        else {
            let PendingFormatUpgrade::FlowPlan { prepared, .. } = pending else {
                unreachable!()
            };
            return self.dispatch_project_save(
                prepared.path().to_owned(),
                StudioDocumentLifecycleCommand::SavePreparedFlowPlan {
                    prepared,
                    capability_confirmed: true,
                },
            );
        };
        self.validate_save_review(&path, &review, Some(&compatibility))
            && self.execute_project_save(path, true)
    }

    fn validate_save_review(
        &mut self,
        path: &std::path::Path,
        review: &SaveTargetReview,
        compatibility: Option<&ProjectSaveCompatibility>,
    ) -> bool {
        if self.project_departure_checkpoint() == review.workspace
            && save_target_fingerprint(path).is_ok_and(|digest| digest == review.target_sha256)
            && self
                .platform_host
                .project_save_compatibility(path)
                .is_ok_and(|current| current.as_ref() == compatibility)
        {
            return true;
        }
        let title = tr(self.locale, "保存目标已失效", "Save confirmation expired");
        let detail = tr(
            self.locale,
            "工程、输入或目标文件已变化或无法核对，请重新检查后保存。",
            "The workspace or target file changed or could not be verified. Review it before saving again.",
        );
        self.cancel_project_save(title, detail);
        false
    }

    fn is_current_project_path(&self, path: &std::path::Path) -> bool {
        self.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .project_path
            .as_deref()
            .is_some_and(|current| paths_match(std::path::Path::new(current), path))
    }

    fn execute_project_save(&mut self, path: PathBuf, upgrade: bool) -> bool {
        let command = if upgrade {
            StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() }
        } else if self.is_current_project_path(&path) {
            StudioDocumentLifecycleCommand::Save
        } else {
            StudioDocumentLifecycleCommand::SaveAs { path: path.clone() }
        };
        self.dispatch_project_save(path, command)
    }

    fn dispatch_project_save(
        &mut self,
        path: PathBuf,
        command: StudioDocumentLifecycleCommand,
    ) -> bool {
        let Some(window_id) = self.current_window_id() else {
            return false;
        };
        let save_as = !self.is_current_project_path(&path);
        let prepared_plan = matches!(
            command,
            StudioDocumentLifecycleCommand::SavePreparedFlowPlan { .. }
        );
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
                true
            }
            Err(error) => {
                self.record_project_save_failure(path, &error);
                if prepared_plan {
                    self.project_open.pending_save_as_overwrite = None;
                }
                false
            }
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
            self.project_open.pending_save_as_overwrite = Some(PendingSaveAs::retry(path.clone()));
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
        let Some(PendingSaveAs { path, action }) =
            self.project_open.pending_save_as_overwrite.take()
        else {
            return;
        };
        match action {
            SaveAsAction::Retry => self.request_save_project_as(path, true),
            SaveAsAction::Overwrite {
                review,
                compatibility,
            } => {
                if !self.validate_save_review(&path, &review, compatibility.as_ref()) {
                    return;
                }
                if let Some(compatibility) = compatibility {
                    self.pending_format_upgrade = Some(PendingFormatUpgrade::Legacy {
                        path,
                        review,
                        compatibility,
                    });
                } else {
                    self.execute_project_save(path, false);
                }
            }
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

    pub(super) fn render_format_upgrade(&mut self, ctx: &egui::Context) -> bool {
        let (path, compatibility) = match self.pending_format_upgrade.as_ref() {
            Some(PendingFormatUpgrade::Legacy {
                path,
                compatibility,
                ..
            }) => (path.display().to_string(), compatibility.clone()),
            Some(PendingFormatUpgrade::FlowPlan {
                prepared,
                overwrite,
            }) => {
                self.render_prepared_flow_plan_save(
                    ctx,
                    prepared.path().display().to_string(),
                    prepared.compatibility().cloned(),
                    *overwrite,
                    prepared.support(),
                );
                return true;
            }
            None => return false,
        };
        let zh = self.locale == StudioShellLocale::ZhCn;
        let response = egui::Modal::new(egui::Id::new("project-format-upgrade")).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 40.0).clamp(220.0, 520.0));
            let capability_only = matches!(compatibility, ProjectSaveCompatibility::RatingCapability | ProjectSaveCompatibility::RatingMatrixCapability | ProjectSaveCompatibility::RatingAdoptionCapability | ProjectSaveCompatibility::FlowCapabilities { .. });
            ui.heading(match (zh, capability_only, &compatibility) {
                (true, _, ProjectSaveCompatibility::FlowCapabilities { watch_list_entries: Some(_), .. }) => "首次保存关注清单",
                (false, _, ProjectSaveCompatibility::FlowCapabilities { watch_list_entries: Some(_), .. }) => "Save watch list for the first time",
                (true, _, ProjectSaveCompatibility::FlowCapabilities { replaces_target_variable_watch_list: true, .. }) => "确认替换目标关注清单",
                (false, _, ProjectSaveCompatibility::FlowCapabilities { replaces_target_variable_watch_list: true, .. }) => "Confirm replacement of destination watch list",
                (true, _, ProjectSaveCompatibility::FlowCapabilities { .. }) => "确认流程能力兼容性",
                (false, _, ProjectSaveCompatibility::FlowCapabilities { .. }) => "Confirm flow capability compatibility",
                (true, true, _) => "确认校核任务兼容性", (false, true, _) => "Confirm rating compatibility",
                (true, false, _) => "保存为新版工程", (false, false, _) => "Upgrade project format",
            });
            match compatibility {
                ProjectSaveCompatibility::FlowCapabilities { required, replaces_target_object_identities, replaces_target_variable_watch_list, watch_list_entries, .. } => {
                    if let Some(count) = watch_list_entries {
                        ui.label(if zh { format!("工程仍为 v4。此次会写入 {count} 项关注引用及其顺序。") } else { format!("The project remains v4. This saves {count} watched references in list order.") });
                        ui.label(tr(self.locale, "不支持关注清单能力的旧版 Studio 将整份只读打开。可另存新副本以保留原件。", "Older Studio versions without watch list support open this project read-only. Use Save As to keep the original."));
                    }
                    if replaces_target_variable_watch_list {
                        ui.strong(tr(self.locale, "目标工程已有的关注清单将被当前清单替换。", "The current watch list will replace the destination project's watch list."));
                    }
                    if replaces_target_object_identities {
                        ui.strong(tr(self.locale, "覆盖后，目标工程的旧变量引用可能失效。", "Overwriting can invalidate old variable references in the destination project."));
                        ui.label(tr(self.locale, "将写入下列必需能力。", "The following required capabilities will be written."));
                    } else if watch_list_entries.is_none() && !replaces_target_variable_watch_list {
                        ui.label(if zh { "工程仍为 v4，将增加所需模型或计划能力。不支持这些能力的旧版本将整份只读打开。可另存为以保留原件。" }
                        else { "The project remains v4 with additional required model or plan capabilities. Older versions without these capabilities open it read-only. Use Save As to keep the original." });
                    }
                    for capability in required { ui.label(format!("{} / {}", capability.id, capability.version)); }
                }
                ProjectSaveCompatibility::RatingAdoptionCapability => {
                    ui.label(if zh { "工程仍为 v3，校核方法升级为 v3 以保存固定采纳证据；不支持采纳的旧版本将整份只读打开。可另存为以保留原件。" }
                    else { "The project remains v3 with rating method v3 to retain fixed adoption evidence. Older versions without adoption support will open the entire project read-only. Use Save As to keep the original." });
                }
                ProjectSaveCompatibility::RatingMatrixCapability => {
                    ui.label(if zh { "工程仍为 v3，校核方法升级为 v2；仅支持单工况校核的旧版本将整份只读打开。可另存为以保留原件。" }
                    else { "The project remains v3 with rating method v2. Versions supporting only single-case rating will open the entire project read-only. Use Save As to keep the original." });
                }
                ProjectSaveCompatibility::RatingCapability => {
                    ui.label(if zh { "工程仍为 v3，新增校核任务后，仅支持初步尺寸的旧版本将只能只读检查。可另存为以保留原件。" }
                    else { "The project remains v3. Older versions that only support sizing will open the rating project read-only. Use Save As to keep the original." });
                }
                ProjectSaveCompatibility::Format { version, includes_rating } => {
                    ui.label(if version == 4 {
                        if zh { "保存为 v4 并声明工程所需的模型与计划能力，保留正式输入和显示设置。早期格式读者将拒绝；不支持所需能力的 v4 程序将整份只读。可另存为以保留原件。".to_string() }
                        else { "Save as v4 with explicit model and plan capabilities, preserving formal inputs and display settings. Earlier format readers reject it; v4 readers without the required capabilities open the whole project read-only. Use Save As to keep the original.".to_string() }
                    } else if zh {
                        format!("此工程将保存为 v{version}，保留正式输入和显示设置，旧版程序无法读取。可另存为以保留原件。")
                    } else {
                        format!("Save as v{version} to preserve formal inputs and display settings. Older versions cannot read it. Use Save As to keep the original.")
                    });
                    if includes_rating {
                        ui.label(if zh { "含校核任务；不支持对应校核方法版本的旧程序将整份只读打开。" }
                        else { "Contains rating tasks; older programs without the required rating method version will open the entire project read-only." });
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
                    self.confirm_save_and_maybe_close(ctx);
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

    fn confirm_save_and_maybe_close(&mut self, ctx: &egui::Context) {
        if self.confirm_format_upgrade()
            && !self
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

    fn render_prepared_flow_plan_save(
        &mut self,
        ctx: &egui::Context,
        path: String,
        compatibility: Option<ProjectSaveCompatibility>,
        overwrite: bool,
        support: rf_store::project_v4::FlowPlanSupport,
    ) {
        use rf_store::project_v4::*;
        let locale = self.locale;
        let (title, capability, version) = match support {
            FlowPlanSupport::SinglePhaseRecycle => (
                tr(
                    locale,
                    "确认保存单相显热回流",
                    "Confirm single-vapor recycle save",
                ),
                SINGLE_PHASE_RECYCLE_CAPABILITY,
                1,
            ),
            FlowPlanSupport::SinglePhase => (
                tr(
                    locale,
                    "确认保存单相显热计划",
                    "Confirm single-vapor plan save",
                ),
                SINGLE_PHASE_CAPABILITY,
                1,
            ),
            FlowPlanSupport::ProductComposition => (
                tr(
                    locale,
                    "确认保存产品组成规格",
                    "Confirm product composition save",
                ),
                EQUATION_ORIENTED_CAPABILITY,
                PRODUCT_COMPOSITION_VERSION,
            ),
            FlowPlanSupport::MixedSerialRecycle => (
                tr(
                    locale,
                    "确认保存串联回流计划",
                    "Confirm serial recycle save",
                ),
                MIXED_SERIAL_RECYCLE_CAPABILITY,
                1,
            ),
            FlowPlanSupport::MixedRecycle => (
                tr(locale, "确认保存混合回流计划", "Confirm mixed recycle save"),
                MIXED_RECYCLE_CAPABILITY,
                1,
            ),
            _ => unreachable!("only capability-specific saves use this review"),
        };
        let response = egui::Modal::new(egui::Id::new("mixed-recycle-save")).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 48.0).clamp(220.0, 500.0));
            ui.heading(title);
            egui::ScrollArea::vertical().max_height((ctx.screen_rect().height() * 0.45).max(70.0)).show(ui, |ui| {
                ui.label(path);
                if overwrite {
                    ui.strong(tr(locale, "将覆盖已有目标文件。", "This replaces the existing destination file."));
                }
                if let Some(compatibility) = compatibility {
                    ui.label(tr(locale, "保存为 v4。不支持所需能力的旧版本将整份只读打开。", "Save as v4. Older versions without the required capabilities open the entire file read-only."));
                    if let ProjectSaveCompatibility::FlowCapabilities { required, replaces_target_object_identities, replaces_target_variable_watch_list, .. } = compatibility {
                        if replaces_target_object_identities {
                            ui.strong(tr(locale, "覆盖可能使目标工程的旧变量引用失效。", "Overwriting can invalidate old variable references."));
                        }
                        if replaces_target_variable_watch_list {
                            ui.strong(tr(locale, "将替换目标工程已有的关注清单。", "This replaces the destination project's watch list."));
                        }
                        for capability in required {
                            ui.label(format!("{} / {}", capability.id, capability.version));
                        }
                    } else {
                        ui.label(format!("{capability} / {version}"));
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                let cancel = ui.button(tr(locale, "取消", "Cancel"));
                let save_as = ui.button(tr(locale, "另存为…", "Save As…"));
                let confirm = ui.button(tr(locale, "继续保存", "Continue and save"));
                if ui.memory(|m| m.focused().is_none())
                    || (!cancel.has_focus() && !save_as.has_focus() && !confirm.has_focus()
                        && !save_as.lost_focus() && !confirm.lost_focus())
                {
                    cancel.request_focus();
                }
                if cancel.clicked() {
                    self.cancel_project_save("Save canceled", "Project file and save baseline preserved.");
                }
                if save_as.clicked() {
                    self.pending_format_upgrade = None;
                    self.save_project_as_from_picker();
                }
                if confirm.clicked() {
                    self.confirm_save_and_maybe_close(ctx);
                }
            });
        });
        if response.should_close() && self.pending_format_upgrade.is_some() {
            self.cancel_project_save("Save canceled", "Project file and save baseline preserved.");
        }
    }
}
