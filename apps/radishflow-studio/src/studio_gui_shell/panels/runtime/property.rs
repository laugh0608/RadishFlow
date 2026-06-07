use super::*;

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn render_runtime_package_tab(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        if let Some(platform_notice) = window.runtime.platform_notice.as_ref() {
            ui.label(egui::RichText::new(self.locale.text(ShellText::PlatformNotice)).strong());
            ui.colored_label(notice_color(platform_notice.level), &platform_notice.title);
            render_wrapped_label(ui, &platform_notice.message);
            ui.separator();
        }

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(self.locale.text(ShellText::PropertyPackage)).strong());
            render_status_chip(
                ui,
                self.locale.runtime_label("Ready").as_ref(),
                egui::Color32::from_rgb(66, 118, 92),
            );
        });
        ui.add_space(4.0);

        let property_page = &window.property_page;
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let current_package_id = property_page
                .selected_package_id
                .as_deref()
                .unwrap_or("unselected");
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(current_package_id).strong());
                render_status_chip(
                    ui,
                    self.locale
                        .runtime_label(property_page.package_status_label)
                        .as_ref(),
                    egui::Color32::from_rgb(66, 118, 92),
                );
            });
            render_wrapped_small(
                ui,
                match self.locale {
                    StudioShellLocale::En => {
                        "Choose the built-in property package stored in the current flowsheet."
                    }
                    StudioShellLocale::ZhCn => "选择写入当前 flowsheet 的内置物性包。",
                },
            );
            ui.add_space(4.0);
            egui::Grid::new("runtime-package-summary")
                .num_columns(2)
                .striped(false)
                .show(ui, |ui| {
                    ui.small(match self.locale {
                        StudioShellLocale::En => "Components",
                        StudioShellLocale::ZhCn => "组分",
                    });
                    let component_summary = property_page
                        .package_choices
                        .iter()
                        .find(|choice| choice.selected)
                        .map(|choice| choice.component_summary.as_str())
                        .unwrap_or("-");
                    ui.small(component_summary);
                    ui.end_row();

                    ui.small(match self.locale {
                        StudioShellLocale::En => "Cache",
                        StudioShellLocale::ZhCn => "缓存",
                    });
                    ui.small(match self.locale {
                        StudioShellLocale::En => "Ready",
                        StudioShellLocale::ZhCn => "就绪",
                    });
                    ui.end_row();

                    ui.small(match self.locale {
                        StudioShellLocale::En => "Example source",
                        StudioShellLocale::ZhCn => "示例来源",
                    });
                    ui.small("examples/flowsheets");
                    ui.end_row();

                    ui.small(match self.locale {
                        StudioShellLocale::En => "Bundled examples",
                        StudioShellLocale::ZhCn => "内置示例",
                    });
                    ui.small(window.home.example_case_tiles.len().to_string());
                    ui.end_row();
                });
        });

        ui.add_space(8.0);
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

        ui.add_space(4.0);
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
                        "Choose from the controlled built-in component catalog stored in the current flowsheet."
                    }
                    StudioShellLocale::ZhCn => {
                        "从受控内置组分目录选择写入当前 flowsheet 的项目组分。"
                    }
                },
            );
            ui.add_space(4.0);
            for component in &property_page.component_choices {
                ui.horizontal(|ui| {
                    let status = if component.selected {
                        self.locale.runtime_label("Selected")
                    } else {
                        self.locale.runtime_label("Available")
                    };
                    ui.label(egui::RichText::new(&component.name).strong());
                    if let Some(formula) = component.formula.as_ref() {
                        ui.small(formula);
                    }
                    ui.small(status.as_ref());
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
                    "Independent property workspace backed by the current flowsheet document."
                }
                StudioShellLocale::ZhCn => "独立物性工作区，状态来自当前 flowsheet 文档。",
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
                ui.columns(3, |columns| {
                    self.render_property_workspace_navigation(&mut columns[0], property_page);
                    self.render_property_workspace_selection(&mut columns[1], property_page);
                    self.render_property_workspace_summary(&mut columns[2], window);
                });
            });
    }

    fn render_property_workspace_navigation(
        &self,
        ui: &mut egui::Ui,
        property_page: &radishflow_studio::StudioGuiWindowPropertyPageModel,
    ) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(match self.locale {
                    StudioShellLocale::En => "Property workspace",
                    StudioShellLocale::ZhCn => "物性工作区",
                })
                .strong(),
            );
            ui.add_space(4.0);
            for label in [
                self.locale.runtime_label("Components"),
                self.locale.runtime_label("Package"),
            ] {
                let _ = ui.selectable_label(false, label.as_ref());
            }
            for section in &property_page.future_sections {
                ui.add_enabled(
                    false,
                    egui::Button::new(self.locale.runtime_label(section).as_ref()),
                );
            }
            ui.separator();
            render_wrapped_small(
                ui,
                match self.locale {
                    StudioShellLocale::En => {
                        "MVP scope exposes controlled built-in packages and a small component catalog."
                    }
                    StudioShellLocale::ZhCn => {
                        "当前 MVP 只暴露受控内置物性包和小型组分目录。"
                    }
                },
            );
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
