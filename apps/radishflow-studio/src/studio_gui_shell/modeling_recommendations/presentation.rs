use super::*;
use rf_types::units::QuantityKind;

pub(super) fn tr(locale: StudioShellLocale, zh: &'static str, en: &'static str) -> &'static str {
    if locale == StudioShellLocale::ZhCn {
        zh
    } else {
        en
    }
}

pub(super) fn action_label(locale: StudioShellLocale, action: &RecommendedAction) -> &'static str {
    match action {
        RecommendedAction::Locate(_) => tr(locale, "定位输入", "Locate input"),
        RecommendedAction::NormalizeComposition(_) => {
            tr(locale, "归一已提交组成", "Normalize committed composition")
        }
        RecommendedAction::MaterialConnection(connection) => match connection.stream {
            rf_ui::CanvasSuggestedStreamBinding::Create { .. } => {
                tr(locale, "创建出口流股", "Create outlet stream")
            }
            rf_ui::CanvasSuggestedStreamBinding::Existing { .. } => {
                tr(locale, "连接到指定入口", "Connect to selected inlet")
            }
        },
    }
}

pub(super) fn accept_label(locale: StudioShellLocale, action: &RecommendedAction) -> &'static str {
    match action {
        RecommendedAction::Locate(_) => tr(locale, "定位输入", "Locate input"),
        RecommendedAction::NormalizeComposition(_) => {
            tr(locale, "接受并归一", "Accept normalization")
        }
        RecommendedAction::MaterialConnection(connection) => match connection.stream {
            rf_ui::CanvasSuggestedStreamBinding::Create { .. } => {
                tr(locale, "接受并创建", "Accept stream creation")
            }
            rf_ui::CanvasSuggestedStreamBinding::Existing { .. } => {
                tr(locale, "接受此连接", "Accept this connection")
            }
        },
    }
}

pub(super) fn action_target(action: &RecommendedAction) -> SpecificationTarget {
    match action {
        RecommendedAction::Locate(target) => target.clone(),
        RecommendedAction::NormalizeComposition(preview) => {
            SpecificationTarget::Composition(preview.stream_id.clone())
        }
        RecommendedAction::MaterialConnection(connection) => {
            SpecificationTarget::Port(rf_types::DiagnosticPortTarget {
                unit_id: connection
                    .sink_unit_id
                    .as_ref()
                    .unwrap_or(&connection.source_unit_id)
                    .clone(),
                port_name: connection
                    .sink_port
                    .as_ref()
                    .unwrap_or(&connection.source_port)
                    .clone(),
            })
        }
    }
}

pub(super) fn target_description(
    action: &RecommendedAction,
    document: &rf_ui::FlowsheetDocument,
) -> String {
    match action {
        RecommendedAction::NormalizeComposition(preview) => document
            .flowsheet
            .streams
            .get(&preview.stream_id)
            .map(|s| format!("{} [{}]", s.name, s.id))
            .unwrap_or_else(|| preview.stream_id.to_string()),
        RecommendedAction::MaterialConnection(c) => {
            let source = format!("{}.{}", c.source_unit_id, c.source_port);
            match (&c.sink_unit_id, &c.sink_port) {
                (Some(id), Some(port)) => format!("{source} → {id}.{port}"),
                _ => source,
            }
        }
        RecommendedAction::Locate(target) => format!("{target:?}"),
    }
}

pub(super) fn rejection_text(locale: StudioShellLocale, error: &RecommendationError) -> String {
    use RecommendationError::*;
    let text = match error {
        PendingDrafts => tr(
            locale,
            "暂停写入：存在待处理输入草稿。定位保留原始文本与单位。",
            "Write paused: input drafts need attention. Navigation preserves their text and units.",
        ),
        PendingCanvasEdit => tr(
            locale,
            "暂停写入：画布仍有待提交编辑。请完成或取消该编辑后重新预览。",
            "Write paused: a canvas edit is pending. Finish or cancel it, then review a new preview.",
        ),
        StaleRevision { .. } => tr(
            locale,
            "预览已过期：正式输入修订已变化。请重新生成并查看预览。",
            "Preview expired: the input revision changed. Generate and review a new preview.",
        ),
        DifferentDocument | DifferentSession => tr(
            locale,
            "预览属于旧工程或旧会话，不能接受。",
            "This preview belongs to another document or session and cannot be accepted.",
        ),
        PreviewExpired | PreviewChanged => tr(
            locale,
            "预览凭据或动作内容已变化。请重新生成并查看预览。",
            "The preview token or action changed. Generate and review a new preview.",
        ),
        Unavailable | Ambiguous => tr(
            locale,
            "没有可唯一匹配的有效候选，请定位输入或查看当前建议。",
            "No uniquely matching valid candidate is available. Locate the input or review current recommendations.",
        ),
        Ignored => tr(
            locale,
            "推荐已在本修订忽略；恢复后需重新预览。",
            "This recommendation is ignored for this revision; restore it and review a new preview.",
        ),
        ChecksUnavailable(_) => tr(
            locale,
            "检查未完成，不能接受旧建议。请重新检查。",
            "Checks are unavailable; old recommendations cannot be accepted. Recheck the inputs.",
        ),
        NavigationOnly => tr(
            locale,
            "此建议只提供定位，不修改输入。",
            "This recommendation only locates an input.",
        ),
        Normalization(_) | Rejected(_) => tr(
            locale,
            "接受被拒绝；工程与历史未提交。请检查输入并重新预览。",
            "Acceptance was rejected; no document or history change was committed. Check the inputs and review a new preview.",
        ),
    };
    match error {
        StaleRevision { expected, actual } => format!("{text} (r{expected} → r{actual})"),
        Rejected(error) => format!("{text} [{}] {}", error.code().as_str(), error.message()),
        _ => text.into(),
    }
}

impl ReadyAppState {
    pub(super) fn recommendation_quantity(&self, value: f64, quantity: QuantityKind) -> String {
        let units = &self
            .platform_host
            .modeling_inputs()
            .workspace
            .project_presentation;
        let unit = units.effective_unit(
            self.current_window_id().map(rf_ui::DisplayUnitViewId),
            quantity,
        );
        rf_ui::format_quantity(value, quantity, unit).unwrap_or_else(|_| {
            format!(
                "{value} {}",
                quantity.definition().canonical_unit.definition().symbol
            )
        })
    }

    pub(super) fn render_recommendation_values(
        &self,
        ui: &mut egui::Ui,
        action: &RecommendedAction,
    ) {
        match action {
            RecommendedAction::NormalizeComposition(preview) => {
                ui.label(tr(
                    self.locale,
                    "组分：已提交 → 接受后（摩尔分数）",
                    "Component: committed → after acceptance (mole fraction)",
                ));
                for (id, before) in &preview.original {
                    ui.label(format!("{id}: {before} → {}", preview.normalized[id]));
                }
                ui.label(format!(
                    "{}: {} → 1",
                    tr(self.locale, "总和", "Total"),
                    preview.sum
                ));
                ui.label(tr(
                    self.locale,
                    "依据 xᵢ / Σx；同时更新全部分量，一次 Undo。",
                    "Uses xᵢ / Σx; all fractions change in one Undo transaction.",
                ));
            }
            RecommendedAction::MaterialConnection(connection) => {
                ui.label(format!(
                    "{}: {}.{}",
                    tr(self.locale, "源端口", "Source port"),
                    connection.source_unit_id,
                    connection.source_port
                ));
                if let (Some(id), Some(port)) = (&connection.sink_unit_id, &connection.sink_port) {
                    ui.label(format!(
                        "{}: {id}.{port}",
                        tr(self.locale, "目标端口", "Target port")
                    ));
                }
                match &connection.stream {
                    rf_ui::CanvasSuggestedStreamBinding::Existing { stream_id } => {
                        ui.label(format!(
                            "{}: {stream_id}",
                            tr(self.locale, "已有流股", "Existing stream")
                        ));
                        ui.label(tr(self.locale, "现有本地规则生成；端口方向、占用和环路在接受前重验。候选排序不代表工程意图。", "Generated by existing local rules; direction, occupancy and cycles are checked again before acceptance. Ordering does not express engineering intent."));
                    }
                    rf_ui::CanvasSuggestedStreamBinding::Create { stream } => {
                        ui.label(format!(
                            "{}: {} [{}]",
                            tr(self.locale, "新流股", "New stream"),
                            stream.name,
                            stream.id
                        ));
                        for (label, quantity, value) in [
                            (
                                tr(self.locale, "温度", "Temperature"),
                                QuantityKind::AbsoluteTemperature,
                                stream.temperature_k,
                            ),
                            (
                                tr(self.locale, "压力", "Pressure"),
                                QuantityKind::AbsolutePressure,
                                stream.pressure_pa,
                            ),
                            (
                                tr(self.locale, "摩尔流量", "Molar flow"),
                                QuantityKind::MolarFlow,
                                stream.total_molar_flow_mol_s,
                            ),
                        ] {
                            ui.label(format!(
                                "{label}: {}",
                                self.recommendation_quantity(value, quantity)
                            ));
                        }
                        ui.label(tr(
                            self.locale,
                            "组成（摩尔分数）",
                            "Composition (mole fractions)",
                        ));
                        if stream.overall_mole_fractions.is_empty() {
                            ui.label(tr(
                                self.locale,
                                "空组成；不会自动添加组分。",
                                "Empty composition; no components will be added.",
                            ));
                        }
                        for (component, value) in &stream.overall_mole_fractions {
                            ui.label(format!("{component}: {value}"));
                        }
                        ui.label(tr(self.locale, "以上为模板初值，接受不代表逐字段人工确认；不补单元显式参数。创建并绑定为一次事务。", "These are template values, not individually confirmed specifications. Unit parameters are not filled in. Creation and binding form one transaction."));
                    }
                }
                ui.small(tr(self.locale, "一次接受递增修订；其他旧候选随之过期，不自动继续连接。", "Acceptance advances the revision; other old candidates expire. No further connection is applied automatically."));
            }
            RecommendedAction::Locate(_) => (),
        }
    }
}
