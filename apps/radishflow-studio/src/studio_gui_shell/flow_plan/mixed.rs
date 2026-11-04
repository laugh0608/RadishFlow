use super::*;
use rf_solver::{equation_oriented::ExplicitEquationScales, mixed::*};
impl ReadyAppState {
    pub(super) fn render_mixed_plan(
        &mut self,
        ui: &mut egui::Ui,
        draft: &mut FlowPlanDraft,
        o: &mut MixedOptions,
    ) -> bool {
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.small(tr(zh,"显式分区：每个单元恰属一块；块间须为 DAG。Local 单单元，EO 仅固定比例正向物料方程。","Explicit partition: each unit belongs to one block; the block graph must be a DAG. Local has one unit; EO supports forward fixed-fraction material equations."));
        let mut apply = false;
        for (field, title, value, unit) in [
            (
                FlowPlanNumericField::Temperature,
                tr(zh, "温度", "Temperature"),
                o.boundary.temperature_k,
                MeasurementUnit::Kelvin,
            ),
            (
                FlowPlanNumericField::Pressure,
                tr(zh, "压力", "Pressure"),
                o.boundary.pressure_pa,
                MeasurementUnit::Pascal,
            ),
        ] {
            apply |= number(
                ui,
                draft,
                field,
                title,
                value,
                unit,
                &mut self.flow_plan.notice,
            );
        }
        let can_change = !draft.has_modified_numbers();
        let flow = self.platform_host.document().flowsheet.clone();
        // The symbolic preview has no invented boundary values and calls no property services.
        let mut symbolic = o.clone();
        for b in &mut symbolic.blocks {
            if let MixedBlockMethod::EquationOriented { scales } = &mut b.method {
                *scales = None;
            }
        }
        let preview = assemble(&flow, &symbolic);
        let mut remove = None;
        for (i, b) in o.blocks.iter_mut().enumerate() {
            ui.push_id(("mixed-block",i),|ui| { egui::Frame::group(ui.style()).show(ui,|ui| {
                ui.strong(format!("{} {}",tr(zh,"分区块","Block"),b.id.as_str()));
                ui.add_enabled_ui(can_change,|ui| {
                    ui.label(tr(zh,"稳定块 ID","Stable block ID"));
                    let mut id = b.id.as_str().to_owned();
                    if ui.add(egui::TextEdit::singleline(&mut id).desired_width(ui.available_width().min(260.0))).changed() { b.id = MixedBlockId::new(id); }
                    let mut eo = matches!(b.method,MixedBlockMethod::EquationOriented { .. });
                    let before = eo;
                    egui::ComboBox::from_id_salt("block-method").selected_text(if eo {"EO"} else {"Local"}).show_ui(ui,|ui| {
                        ui.selectable_value(&mut eo,false,tr(zh,"Local · 单单元实际调用","Local · actual single-unit call"));
                        ui.selectable_value(&mut eo,true,tr(zh,"EO · 正向物料方程","EO · forward material equations"));
                    });
                    if eo != before { b.method = if eo {MixedBlockMethod::EquationOriented { scales: None }} else {MixedBlockMethod::Local}; }
                    ui.collapsing(tr(zh,"选择成员（原工程身份）","Select members (original identities)"),|ui| {
                        for (id,u) in &flow.units {
                            let mut selected = b.members.contains(id);
                            if ui.checkbox(&mut selected,format!("{} · {id} · {}",u.name,u.kind)).changed() {
                                if selected { b.members.insert(id.clone()); } else { b.members.remove(id); }
                            }
                        }
                    });
                });
                for id in b.members.clone() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{id}{}",if flow.units.contains_key(&id) {""} else {tr(zh," · 绑定失效，保留身份"," · binding unavailable, identity retained")}));
                        if ui.small_button(tr(zh,"定位","Locate")).clicked() { self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(&rf_ui::InspectorTarget::Unit(id.clone()))); }
                        if ui.add_enabled(can_change,egui::Button::new(tr(zh,"移除成员","Remove member"))).clicked() { b.members.remove(&id); }
                    });
                }
                if let MixedBlockMethod::EquationOriented { scales } = &mut b.method {
                    let layout = preview.as_ref().ok().and_then(|p| p.equation_layout(&b.id).ok());
                    if let Some(l) = &layout { ui.label(format!("{} {} / {} {}",l.variables.len(),tr(zh,"变量","variables"),l.equations.len(),tr(zh,"方程","equations"))); }
                    ui.small(tr(zh,"仅符号布局；真实边界到达后才检查数值可解性。","Symbolic layout only; numerical solvability is checked after actual boundaries arrive."));
                    if scales.is_none() {
                        ui.label(tr(zh,"默认尺度：本块实际新鲜 / 入界总量。","Default scales: actual fresh / incoming total in this block."));
                        if ui.add_enabled(can_change && layout.is_some(),egui::Button::new(tr(zh,"生成完整显式尺度草稿（1 mol/s）","Create complete explicit scale draft (1 mol/s)"))).clicked() {
                            let l = layout.as_ref().expect("enabled layout");
                            *scales = Some(ExplicitEquationScales { variables:l.variables.iter().cloned().map(|v|(v,1.0)).collect(),equations:l.equations.iter().cloned().map(|e|(e,1.0)).collect() });
                        }
                    } else if ui.add_enabled(can_change,egui::Button::new(tr(zh,"恢复默认尺度规则","Use default scale rule"))).clicked() { *scales = None; }
                    if let Some(s) = scales {
                        ui.collapsing(tr(zh,"完整显式变量 / 残差尺度","Complete explicit variable / residual scales"),|ui| {
                            for (binding,v) in &s.variables {
                                ui.label(format!("{binding:?}"));
                                apply |= number(ui,draft,FlowPlanNumericField::MixedVariableScale { block:b.id.clone(),binding:binding.clone() },tr(zh,"变量尺度","Variable scale"),*v,MeasurementUnit::MolePerSecond,&mut self.flow_plan.notice);
                            }
                            for (binding,v) in &s.equations {
                                ui.label(format!("{binding:?}"));
                                apply |= number(ui,draft,FlowPlanNumericField::MixedResidualScale { block:b.id.clone(),binding:binding.clone() },tr(zh,"残差尺度","Residual scale"),*v,MeasurementUnit::MolePerSecond,&mut self.flow_plan.notice);
                            }
                        });
                    }
                }
                if ui.add_enabled(can_change,egui::Button::new(tr(zh,"删除此块","Delete block"))).clicked() {remove = Some(i);}
            }); });
        }
        if let Some(i) = remove {
            o.blocks.remove(i);
        }
        if ui
            .add_enabled(
                can_change,
                egui::Button::new(tr(zh, "添加分区块", "Add block")),
            )
            .clicked()
        {
            let mut index = 1;
            while o
                .blocks
                .iter()
                .any(|b| b.id.as_str() == format!("block-{index}"))
            {
                index += 1;
            }
            o.blocks.push(MixedBlock {
                id: MixedBlockId::new(format!("block-{index}")),
                members: Default::default(),
                method: MixedBlockMethod::Local,
            });
        }
        match assemble(&flow, o) {
            Ok(p) => {
                ui.label(format!(
                    "DAG: {}",
                    p.blocks()
                        .iter()
                        .map(|b| b.id.as_str())
                        .collect::<Vec<_>>()
                        .join(" → ")
                ));
            }
            Err(e) => {
                ui.colored_label(
                    egui::Color32::DARK_RED,
                    format!("{}: {e}", tr(zh, "分区尚未就绪", "Partition not ready")),
                );
                for id in e.context().related_unit_ids() {
                    if ui
                        .button(format!("{} {id}", tr(zh, "定位单元", "Locate unit")))
                        .clicked()
                    {
                        self.dispatch_ui_command(format!("inspector.focus_unit:{id}"));
                    }
                }
                for id in e.context().related_stream_ids() {
                    if ui
                        .button(format!("{} {id}", tr(zh, "定位流股", "Locate stream")))
                        .clicked()
                    {
                        self.dispatch_ui_command(format!("inspector.focus_stream:{id}"));
                    }
                }
            }
        }
        ui.small(tr(zh,"不自动合并、补齐成员或切换算法。保存仅写正式配置；重开后结果为 Missing，需显式重跑。","No automatic merge, member repair or method switch. Save writes formal configuration; reopen yields Missing and requires an explicit rerun."));
        apply
    }
}
