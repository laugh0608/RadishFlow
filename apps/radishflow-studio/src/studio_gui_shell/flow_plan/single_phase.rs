use super::*;
use rf_solver::flow_plan::SinglePhaseOptions;
use rf_solver::single_phase_recycle::{SinglePhaseRecycleOptions, SinglePhaseTearState};
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{ComponentId, UnitId};

pub(super) struct ThermalUnit {
    id: UnitId,
    name: String,
    outlet_temperature_k: Option<f64>,
}

impl ReadyAppState {
    pub(super) fn render_single_phase_recycle_options(
        &mut self,
        ui: &mut egui::Ui,
        draft: &mut FlowPlanDraft,
        options: &mut SinglePhaseRecycleOptions,
        components: &[ComponentId],
        bindings: &[SinglePhaseTearBinding],
    ) -> bool {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let structural_edit = !draft.has_modified_numbers();
        ui.label(tr(zh, "撕裂流股", "Tear stream"));
        ui.add_enabled_ui(structural_edit, |ui| {
            let picker = egui::ComboBox::from_id_salt("single-phase-recycle-tear")
                .selected_text(if options.tear.stream_id.as_str().is_empty() {
                    tr(zh, "选择撕裂流股…", "Choose a tear stream…")
                } else {
                    options.tear.stream_id.as_str()
                })
                .show_ui(ui, |ui| {
                    if bindings.is_empty() {
                        ui.label(tr(
                            zh,
                            "将 Splitter 的 outlet_a 连至循环 Mixer 的入口。",
                            "Connect Splitter outlet_a to a recycle Mixer inlet.",
                        ));
                    }
                    for binding in bindings {
                        let label = format!(
                            "{} · {}.{} → {}.{}",
                            binding.stream_id,
                            binding.source.unit_id,
                            binding.source.port_name,
                            binding.sink.unit_id,
                            binding.sink.port_name
                        );
                        if ui
                            .selectable_label(&options.tear == binding, label)
                            .clicked()
                            && &options.tear != binding
                        {
                            options.tear = binding.clone();
                            options.initial_state = None;
                        }
                    }
                });
            reveal_plan_focus(&picker.response);
        });
        if options.tear.stream_id.as_str().is_empty() {
            return false;
        }
        ui.label(format!(
            "{}.{} → {}.{}",
            options.tear.source.unit_id,
            options.tear.source.port_name,
            options.tear.sink.unit_id,
            options.tear.sink.port_name
        ));
        let mut submit = number(
            ui,
            draft,
            FlowPlanNumericField::IterationBudget,
            tr(zh, "迭代预算", "Iteration budget"),
            options.max_iteration_evaluations as f64,
            MeasurementUnit::One,
            &mut self.flow_plan.notice,
        )
        .is_some();
        let mut explicit = options.initial_state.is_some();
        ui.add_enabled_ui(structural_edit, |ui| {
            let toggle = ui.checkbox(
                &mut explicit,
                tr(zh, "显式撕裂初值", "Specify tear initial state"),
            );
            reveal_plan_focus(&toggle);
        });
        if !explicit {
            options.initial_state = None;
            ui.small(tr(
                zh,
                "零组分流量与零焓流初值。",
                "Zero initial component and enthalpy flows.",
            ));
            return submit;
        }
        let initial = options
            .initial_state
            .get_or_insert_with(|| SinglePhaseTearState {
                component_flows_mol_s: components.iter().cloned().map(|id| (id, 0.0)).collect(),
                enthalpy_flow_w: 0.0,
            });
        for (component, value) in &initial.component_flows_mol_s {
            submit |= number(
                ui,
                draft,
                FlowPlanNumericField::InitialComponent {
                    stream: options.tear.stream_id.clone(),
                    component: component.clone(),
                },
                &format!("{component} · mol/s"),
                *value,
                MeasurementUnit::MolePerSecond,
                &mut self.flow_plan.notice,
            )
            .is_some();
        }
        submit |= number(
            ui,
            draft,
            FlowPlanNumericField::InitialEnthalpyFlow(options.tear.stream_id.clone()),
            tr(zh, "初始焓流 (W)", "Initial enthalpy flow (W)"),
            initial.enthalpy_flow_w,
            MeasurementUnit::One,
            &mut self.flow_plan.notice,
        )
        .is_some();
        submit
    }

    pub(super) fn single_phase_thermal_units(&self) -> Vec<ThermalUnit> {
        self.platform_host
            .modeling_inputs()
            .workspace
            .document
            .flowsheet
            .units
            .values()
            .filter(|u| matches!(u.kind.as_str(), "heater" | "cooler"))
            .map(|u| ThermalUnit {
                id: u.id.clone(),
                name: u.name.clone(),
                outlet_temperature_k: u.parameters.outlet_temperature_k,
            })
            .collect()
    }

    pub(super) fn render_single_phase_plan(
        &mut self,
        ui: &mut egui::Ui,
        draft: &mut FlowPlanDraft,
        options: &mut SinglePhaseOptions,
        components: &[ComponentId],
        units: &[ThermalUnit],
        applied: bool,
    ) -> (bool, Option<UnitId>) {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let (min_k, max_k) = options.model.temperature_range_k().into_inner();
        let pressure = options.model.pressure_pa();
        let mut apply = false;
        let mut clear = None;
        ui.small(tr(zh, "气相常热容，298.15 K 参考态。输入适用温区与固定压力。", "Constant vapor heat capacity, referenced to 298.15 K. Enter the valid temperature range and fixed pressure."));
        for (field, label, value, unit) in [
            (
                FlowPlanNumericField::SinglePhaseMinimumTemperature,
                tr(zh, "最低温度", "Minimum temperature"),
                min_k,
                MeasurementUnit::Kelvin,
            ),
            (
                FlowPlanNumericField::SinglePhaseMaximumTemperature,
                tr(zh, "最高温度", "Maximum temperature"),
                max_k,
                MeasurementUnit::Kelvin,
            ),
            (
                FlowPlanNumericField::SinglePhasePressure,
                tr(zh, "固定压力", "Fixed pressure"),
                pressure,
                MeasurementUnit::Pascal,
            ),
        ] {
            apply |= number(
                ui,
                draft,
                field,
                label,
                value,
                unit,
                &mut self.flow_plan.notice,
            )
            .is_some();
        }
        let mut capacities = options.model.heat_capacities_j_per_mol_k().clone();
        let capacity_ids: std::collections::BTreeSet<_> = components
            .iter()
            .chain(capacities.keys())
            .cloned()
            .collect();
        ui.strong(tr(zh, "气相摩尔热容", "Vapor molar heat capacity"));
        for id in capacity_ids {
            ui.push_id(&id, |ui| {
                let field = FlowPlanNumericField::VaporHeatCapacity(id.clone());
                if let Some(value) = capacities.get(&id) {
                    apply |= number(
                        ui,
                        draft,
                        field,
                        &format!("{id} · J/(mol·K)"),
                        *value,
                        MeasurementUnit::One,
                        &mut self.flow_plan.notice,
                    )
                    .is_some();
                    if ui
                        .small_button(tr(zh, "移除热容", "Remove heat capacity"))
                        .clicked()
                    {
                        capacities.remove(&id);
                    }
                } else if ui
                    .small_button(format!(
                        "{} · {id}",
                        tr(zh, "添加热容", "Add heat capacity")
                    ))
                    .clicked()
                {
                    capacities.insert(id.clone(), 1.0);
                    draft
                        .edit_number(field, 1.0, MeasurementUnit::One)
                        .expect("finite placeholder")
                        .raw
                        .clear();
                }
            });
        }
        if &capacities != options.model.heat_capacities_j_per_mol_k() {
            options.model = SensibleEnthalpyModel::new(capacities, min_k..=max_k, pressure)
                .expect("validated draft structure");
        }

        if !units.is_empty() || !options.heat_duties_w.is_empty() {
            ui.strong(tr(
                zh,
                "Heater／Cooler 规格",
                "Heater / Cooler specifications",
            ));
        }
        for unit in units {
            ui.push_id(&unit.id, |ui| {
                ui.label(&unit.name);
                if let Some(duty) = options.heat_duties_w.get(&unit.id).copied() {
                    apply |= number(
                        ui,
                        draft,
                        FlowPlanNumericField::HeatDuty(unit.id.clone()),
                        tr(zh, "指定热负荷 (W)", "Specified heat duty (W)"),
                        duty,
                        MeasurementUnit::One,
                        &mut self.flow_plan.notice,
                    )
                    .is_some();
                    if ui
                        .small_button(tr(zh, "移除热负荷", "Remove heat duty"))
                        .clicked()
                    {
                        options.heat_duties_w.remove(&unit.id);
                    }
                }
                if let Some(t) = unit.outlet_temperature_k {
                    ui.label(format!(
                        "{} {t} K",
                        tr(zh, "指定出口温度", "Specified outlet temperature")
                    ));
                    if applied
                        && ui
                            .small_button(tr(zh, "清除出口温度", "Clear outlet temperature"))
                            .clicked()
                    {
                        clear = Some(unit.id.clone());
                    }
                } else if !options.heat_duties_w.contains_key(&unit.id)
                    && ui
                        .small_button(tr(zh, "指定热负荷", "Specify heat duty"))
                        .clicked()
                {
                    options.heat_duties_w.insert(unit.id.clone(), 0.0);
                    draft
                        .edit_number(
                            FlowPlanNumericField::HeatDuty(unit.id.clone()),
                            0.0,
                            MeasurementUnit::One,
                        )
                        .expect("finite placeholder")
                        .raw
                        .clear();
                }
                if unit.outlet_temperature_k.is_none()
                    && !options.heat_duties_w.contains_key(&unit.id)
                {
                    ui.small(tr(
                        zh,
                        "可在模块设置提交出口温度。",
                        "Set an outlet temperature in Unit Inspector.",
                    ));
                }
            });
        }
        options.heat_duties_w.retain(|id, _| {
            if units.iter().any(|unit| &unit.id == id) {
                return true;
            }
            !ui.small_button(format!(
                "{} · {id}",
                tr(zh, "移除失效热负荷", "Remove dangling duty")
            ))
            .clicked()
        });
        (apply, clear)
    }
}
