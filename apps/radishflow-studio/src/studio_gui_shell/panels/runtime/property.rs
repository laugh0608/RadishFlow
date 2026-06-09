use super::*;

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn render_property_workspace_page(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        let property_page = &window.property_page;
        ui.horizontal_wrapped(|ui| {
            ui.heading(self.locale.runtime_label(property_page.title).as_ref());
            render_status_chip(
                ui,
                self.locale
                    .runtime_label(property_page.package_status_label)
                    .as_ref(),
                egui::Color32::from_rgb(66, 118, 92),
            );
            ui.small(format!(
                "{}: {}",
                self.locale.runtime_label("Components"),
                property_page.selected_component_count
            ));
        });
        render_wrapped_small(
            ui,
            match self.locale {
                StudioShellLocale::En => {
                    "Property page state is backed by the current flowsheet document."
                }
                StudioShellLocale::ZhCn => "物性页状态来自当前 flowsheet 文档。",
            },
        );

        if let Some(platform_notice) = window.runtime.platform_notice.as_ref() {
            ui.add_space(6.0);
            ui.colored_label(notice_color(platform_notice.level), &platform_notice.title);
            render_wrapped_label(ui, &platform_notice.message);
        }

        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt(format!(
                "scroll:{}:property-page",
                window.layout_state.scope.layout_key
            ))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    self.render_property_workspace_selection(&mut columns[0], property_page);
                    self.render_property_workspace_summary(&mut columns[1], window);
                });
            });
    }

    fn render_property_workspace_selection(
        &mut self,
        ui: &mut egui::Ui,
        property_page: &radishflow_studio::StudioGuiWindowPropertyPageModel,
    ) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(self.locale.text(ShellText::PropertyPackage)).strong());
            render_wrapped_small(
                ui,
                match self.locale {
                    StudioShellLocale::En => {
                        "Choose the built-in package written to the current flowsheet."
                    }
                    StudioShellLocale::ZhCn => "选择写入当前 flowsheet 的内置物性包。",
                },
            );
            ui.add_space(6.0);
            for choice in &property_page.package_choices {
                let label = property_package_choice_label(self.locale, choice);
                let response = ui
                    .add_enabled(
                        choice.enabled,
                        egui::Button::new(label)
                            .selected(choice.selected)
                            .min_size(egui::vec2(ui.available_width(), 30.0)),
                    )
                    .on_hover_text(&choice.detail);
                render_wrapped_small(ui, &choice.package_id);
                render_wrapped_small(ui, &choice.component_summary);
                if response.clicked() {
                    self.dispatch_ui_command(&choice.command_id);
                }
                ui.add_space(6.0);
            }
        });

        ui.add_space(8.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(match self.locale {
                    StudioShellLocale::En => "Project components",
                    StudioShellLocale::ZhCn => "项目组分",
                })
                .strong(),
            );
            render_wrapped_small(
                ui,
                match self.locale {
                    StudioShellLocale::En => {
                        "Select project components from the controlled built-in catalog."
                    }
                    StudioShellLocale::ZhCn => "从受控内置目录选择项目组分。",
                },
            );
            ui.add_space(6.0);
            for component in &property_page.component_choices {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(&component.name).strong());
                    if let Some(formula) = component.formula.as_ref() {
                        ui.small(formula);
                    }
                    render_status_chip(
                        ui,
                        self.locale
                            .runtime_label(if component.selected {
                                "Selected"
                            } else {
                                "Available"
                            })
                            .as_ref(),
                        if component.selected {
                            egui::Color32::from_rgb(66, 118, 92)
                        } else {
                            egui::Color32::from_rgb(86, 96, 108)
                        },
                    );
                });

                if component.selected {
                    let remove_label = match self.locale {
                        StudioShellLocale::En => "Remove",
                        StudioShellLocale::ZhCn => "移除",
                    };
                    if ui
                        .add_enabled(
                            component.remove_enabled,
                            egui::Button::new(remove_label)
                                .min_size(egui::vec2(ui.available_width(), 28.0)),
                        )
                        .on_hover_text(&component.remove_detail)
                        .clicked()
                    {
                        self.dispatch_ui_command(&component.remove_command_id);
                    }
                } else {
                    let select_label = match self.locale {
                        StudioShellLocale::En => "Select",
                        StudioShellLocale::ZhCn => "选择",
                    };
                    if ui
                        .button(select_label)
                        .on_hover_text(&component.component_id)
                        .clicked()
                    {
                        self.dispatch_ui_command(&component.select_command_id);
                    }
                }
                render_wrapped_small(ui, &component.component_id);
                ui.add_space(6.0);
            }
        });
    }

    fn render_property_workspace_summary(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        let property_page = &window.property_page;
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(match self.locale {
                    StudioShellLocale::En => "Summary",
                    StudioShellLocale::ZhCn => "摘要",
                })
                .strong(),
            );
            for metric in &property_page.metrics {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(self.locale.runtime_label(metric.label).as_ref())
                            .strong(),
                    );
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(&metric.value).as_ref(),
                        egui::Color32::from_rgb(86, 118, 168),
                    );
                });
                render_wrapped_small(ui, &metric.detail);
            }
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                render_status_chip(
                    ui,
                    self.locale
                        .runtime_label(property_page.flowsheet_modeling_status_label)
                        .as_ref(),
                    property_modeling_status_color(property_page.flowsheet_modeling_status_label),
                );
                if ui
                    .add_enabled(
                        property_page.flowsheet_modeling_enabled,
                        egui::Button::new(
                            self.locale
                                .runtime_label("Enter Flowsheet Modeling")
                                .as_ref(),
                        ),
                    )
                    .on_hover_text(
                        self.locale
                            .runtime_label(&property_page.flowsheet_modeling_detail)
                            .as_ref(),
                    )
                    .clicked()
                {
                    self.enter_flowsheet_modeling_from_property();
                }
            });
            render_wrapped_small(
                ui,
                self.locale
                    .runtime_label(&property_page.flowsheet_modeling_detail)
                    .as_ref(),
            );
        });

        if let Some(entitlement_host) = window.runtime.entitlement_host.as_ref() {
            let entitlement = &entitlement_host.presentation.panel.view;
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new(self.locale.text(ShellText::Platform)).strong());
                render_wrapped_small(
                    ui,
                    format!(
                        "{}: {}",
                        self.locale.text(ShellText::AllowedPackages),
                        entitlement.allowed_package_count
                    ),
                );
                render_wrapped_small(
                    ui,
                    format!(
                        "{}: {}",
                        self.locale.text(ShellText::CachedManifests),
                        entitlement.package_manifest_count
                    ),
                );
                if let Some(notice) = entitlement.notice.as_ref() {
                    ui.add_space(4.0);
                    ui.colored_label(notice_color_from_entitlement(notice.level), &notice.title);
                    render_wrapped_label(ui, &notice.message);
                }
            });
        }
    }
}

fn property_package_choice_label(
    locale: StudioShellLocale,
    choice: &radishflow_studio::StudioGuiWindowPropertyPackageModel,
) -> String {
    match locale {
        StudioShellLocale::En => choice.label.clone(),
        StudioShellLocale::ZhCn => match choice.package_id.as_str() {
            "binary-hydrocarbon-lite-v1" => "二元烃 Lite".to_string(),
            _ => choice.label.clone(),
        },
    }
}

fn property_modeling_status_color(status_label: &str) -> egui::Color32 {
    match status_label {
        "Ready" => egui::Color32::from_rgb(54, 128, 84),
        "Incomplete" => egui::Color32::from_rgb(180, 120, 20),
        _ => run_status_color(status_label),
    }
}
