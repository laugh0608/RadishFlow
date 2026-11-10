use super::*;
impl ReadyAppState {
    pub(super) fn render_exact_connection(&mut self, ui: &mut egui::Ui, access: FlowEditAccess) {
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.collapsing(tr(zh,"建模：Splitter 与显式连接","Modeling and connections"), |ui| {
            ui.add_enabled_ui(access == FlowEditAccess::Editable, |ui| {
                if ui.button(tr(zh,"放置 Splitter","Place Splitter")).clicked() { self.dispatch_ui_command("canvas.begin_place_unit.splitter"); self.screen = StudioShellScreen::Workbench; }
                let workspace = &self.platform_host.modeling_inputs().workspace;
                let expected = FlowEditContext::capture(workspace);
                let mut sources = Vec::new(); let mut sinks = Vec::new();
                for unit in workspace.document.flowsheet.units.values() { for port in unit.ports.iter().filter(|p| p.kind == PortKind::Material) {
                    let target = MaterialPortTarget { unit_id: unit.id.clone(), port: port.name.clone() };
                    match port.direction { PortDirection::Outlet => sources.push(target), PortDirection::Inlet => sinks.push(target) }
                }}
                ui.label(tr(zh,"明确选择源端口与汇端口；允许闭环，不替换已占用端口。","Select source and sink explicitly. Cycles are allowed; occupied ports are not replaced."));
                for (title, selection, ports, id) in [(tr(zh,"源端口","Source port"),&mut self.flow_plan.source,&sources,"exact-source"),(tr(zh,"汇端口","Sink port"),&mut self.flow_plan.sink,&sinks,"exact-sink")] {
                    ui.label(title);
                    egui::ComboBox::from_id_salt(id).selected_text(selection.as_ref().map(|p| format!("{}.{}",p.unit_id,p.port)).unwrap_or_else(|| "—".into())).show_ui(ui, |ui| { for port in ports { ui.selectable_value(selection,Some(port.clone()),format!("{}.{}",port.unit_id,port.port)); } });
                }
                ui.label(tr(zh,"流股 ID（现有流股或新的显式身份）","Stream ID (existing stream or new explicit identity)"));
                ui.add(egui::TextEdit::singleline(&mut self.flow_plan.stream).desired_width(ui.available_width().min(300.0)));
                if ui.add_enabled(self.flow_plan.source.is_some() && self.flow_plan.sink.is_some() && !self.flow_plan.stream.trim().is_empty(),egui::Button::new(tr(zh,"确认连接","Confirm connection"))).clicked() {
                    let target = MaterialConnectionTarget { stream_id: StreamId::new(self.flow_plan.stream.clone()), source: self.flow_plan.source.clone().expect("enabled source"), sink: self.flow_plan.sink.clone() };
                    self.plan_action(Action::EditFlow { access, request: FlowActionRequest { expected, action: FlowAction::ConnectExact(target) } });
                }
            });
        });
    }
}
