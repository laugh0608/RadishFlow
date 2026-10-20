use super::*;

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn render_problem_recommendations(
        &mut self,
        ui: &mut egui::Ui,
        problem: &SpecificationProblemKey,
    ) {
        let entries = self
            .modeling_problems
            .recommendations
            .entries()
            .iter()
            .filter(|e| {
                e.recommendation.related_problems.contains(problem)
                    && !matches!(e.recommendation.action, RecommendedAction::Locate(_))
            })
            .cloned()
            .collect::<Vec<_>>();
        if entries.is_empty() {
            ui.small(tr(self.locale, "现有规则未生成可执行推荐，请定位输入或编辑连接。", "The current rules provide no executable recommendation. Locate the input or edit its connections."));
        }
        for entry in entries {
            self.render_recommendation_entry(ui, &entry);
        }
    }

    pub(in crate::studio_gui_shell) fn render_modeling_recommendations(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        self.refresh_modeling_problems();
        let entries = self
            .modeling_problems
            .recommendations
            .entries()
            .iter()
            .filter(|e| !matches!(e.recommendation.action, RecommendedAction::Locate(_)))
            .cloned()
            .collect::<Vec<_>>();
        let ignored = entries
            .iter()
            .filter(|e| e.visibility == RecommendationVisibility::Ignored)
            .count();
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!(
                "{} · {}",
                tr(self.locale, "可执行推荐", "Actionable recommendations"),
                entries.len() - ignored
            ));
            ui.checkbox(
                &mut self.modeling_problems.recommendation_ui.show_ignored,
                format!(
                    "{} · {ignored}",
                    tr(self.locale, "显示已忽略", "Show ignored")
                ),
            );
            self.render_modeling_draft_access(ui);
        });
        self.render_recommendation_notice(ui);
        if entries.is_empty() {
            ui.label(tr(self.locale, "当前规则没有可执行候选。可到当前问题定位输入或编辑连接。", "No executable candidate is available under the current rules. Locate inputs from Current problems or edit connections."));
        }
        for entry in entries {
            if entry.visibility == RecommendationVisibility::Available
                || self.modeling_problems.recommendation_ui.show_ignored
            {
                self.render_recommendation_entry(ui, &entry);
            }
        }
    }

    fn render_recommendation_entry(&mut self, ui: &mut egui::Ui, entry: &RecommendationEntry) {
        ui.push_id(
            format!("recommendation:{:?}", entry.recommendation.key),
            |ui| {
                ui.separator();
                ui.label(target_description(
                    &entry.recommendation.action,
                    self.platform_host.document(),
                ));
                let ignored = entry.visibility == RecommendationVisibility::Ignored;
                if ignored {
                    ui.label(tr(
                        self.locale,
                        "本修订已忽略；问题与运行阻断不变。",
                        "Ignored for this revision; problems and run blockers are unchanged.",
                    ));
                }
                ui.horizontal_wrapped(|ui| {
                    let label = if ignored {
                        tr(self.locale, "恢复推荐", "Restore recommendation").to_owned()
                    } else {
                        format!(
                            "{}：{}",
                            tr(self.locale, "预览", "Preview"),
                            action_label(self.locale, &entry.recommendation.action)
                        )
                    };
                    if ui.add(egui::Button::new(label).wrap()).clicked() {
                        consume_recommendation_activation(ui);
                        if ignored {
                            self.ignore_modeling_recommendation(&entry.recommendation.key, false);
                        } else {
                            self.select_recommendation_problem(&entry.recommendation.key);
                            self.preview_modeling_recommendation(&entry.recommendation.key);
                        }
                    }
                    if !ignored
                        && ui
                            .button(tr(self.locale, "本修订忽略", "Ignore this revision"))
                            .clicked()
                    {
                        consume_recommendation_activation(ui);
                        self.ignore_modeling_recommendation(&entry.recommendation.key, true);
                    }
                });
            },
        );
    }

    pub(in crate::studio_gui_shell) fn render_recommendation_notice(&self, ui: &mut egui::Ui) {
        if let Some(error) = &self.modeling_problems.recommendation_ui.notice {
            ui.label(rejection_text(self.locale, error));
        }
    }

    pub(in crate::studio_gui_shell) fn render_recommendation_review(
        &mut self,
        ui: &mut egui::Ui,
    ) -> bool {
        let Some(review) = &self.modeling_problems.recommendation_ui.review else {
            return false;
        };
        let preview = review.preview.clone();
        let outcome = review.outcome.clone();
        let recommendation = preview.recommendation();
        let committed = matches!(outcome, Some(Ok(_)));
        let validation = if committed {
            Ok(())
        } else {
            self.modeling_problems
                .recommendations
                .validate_preview(self.platform_host.modeling_inputs(), &preview)
        };
        let frame = ui.ctx().cumulative_pass_nr();
        let state = self
            .modeling_problems
            .recommendation_ui
            .review
            .as_mut()
            .expect("review retained");
        let just_opened = state.first_frame.is_none();
        let first_frame = *state.first_frame.get_or_insert(frame);
        if validation.is_err() {
            state.requires_regeneration = true;
        }
        let needs_new = state.requires_regeneration;
        ui.strong(format!(
            "{} · {}",
            tr(self.locale, "预览", "Preview"),
            action_label(self.locale, &recommendation.action)
        ));
        ui.label(target_description(
            &recommendation.action,
            self.platform_host.document(),
        ));
        ui.small(format!(
            "{} r{} · {} r{}",
            tr(self.locale, "预览依据", "Preview input"),
            recommendation.identity.input_revision,
            tr(self.locale, "当前输入", "Current input"),
            self.platform_host.document().revision
        ));
        if let Some(Ok(receipt)) = &outcome {
            ui.strong(format!(
                "✓ {} · r{} · {}",
                tr(self.locale, "动作已执行", "Action committed"),
                receipt.revision,
                tr(self.locale, "一次 Undo", "One Undo")
            ));
            if matches!(
                self.modeling_problems.checks().phase(),
                ModelingCheckPhase::Failed(_)
            ) {
                ui.label(tr(self.locale, "重检失败；当前问题数未知。动作已提交，不要重复接受。", "Recheck failed; the current problem count is unknown. The action is committed; do not accept it again."));
                if ui.button(tr(self.locale, "重新检查", "Recheck")).clicked() {
                    consume_recommendation_activation(ui);
                    self.modeling_problems
                        .recommendations
                        .recheck(self.platform_host.modeling_inputs());
                }
            } else {
                ui.label(tr(self.locale, "问题结论以当前检查报告为准；其他问题可能仍存在。", "Problem status comes from the current check report; other problems may remain."));
            }
        } else {
            if let Some(Err(error)) = &outcome {
                ui.label(rejection_text(self.locale, error));
            }
            if let Err(error) = &validation {
                ui.label(rejection_text(self.locale, error));
                if matches!(error, RecommendationError::PendingDrafts) {
                    self.render_modeling_draft_access(ui);
                }
            } else if needs_new {
                ui.label(tr(
                    self.locale,
                    "阻挡条件已处理，请重新生成并查看预览。",
                    "The blocking condition has cleared. Generate and review a new preview.",
                ));
            }
        }
        self.render_recommendation_values(ui, &recommendation.action);
        ui.horizontal_wrapped(|ui| {
            let back = ui.add(
                egui::Button::new(tr(self.locale, "返回，不再执行", "Back without executing"))
                    .wrap(),
            );
            if just_opened {
                back.request_focus();
            } else {
                reveal_focused_action(&back);
            }
            if back.clicked() {
                consume_recommendation_activation(ui);
                self.close_recommendation_review();
            }
            if !committed {
                if validation.is_err() || needs_new || outcome.is_some() {
                    let regenerate = ui.add(
                        egui::Button::new(tr(
                            self.locale,
                            "重新生成并查看预览",
                            "Generate and review preview",
                        ))
                        .wrap(),
                    );
                    reveal_focused_action(&regenerate);
                    if regenerate.clicked() {
                        consume_recommendation_activation(ui);
                        self.preview_modeling_recommendation(&recommendation.key);
                    }
                }
                let enabled =
                    validation.is_ok() && !needs_new && outcome.is_none() && frame > first_frame;
                let accept = ui.add_enabled(
                    enabled,
                    egui::Button::new(accept_label(self.locale, &recommendation.action)).wrap(),
                );
                reveal_focused_action(&accept);
                if accept.clicked() {
                    consume_recommendation_activation(ui);
                    self.accept_modeling_recommendation();
                }
                let ignore = ui.button(tr(self.locale, "本修订忽略", "Ignore this revision"));
                reveal_focused_action(&ignore);
                if ignore.clicked() {
                    consume_recommendation_activation(ui);
                    self.ignore_modeling_recommendation(&recommendation.key, true);
                }
            }
            let target = action_target(&recommendation.action);
            let locate = ui.add_enabled(
                target.exists(self.platform_host.document()),
                egui::Button::new(tr(self.locale, "定位输入", "Locate input")),
            );
            reveal_focused_action(&locate);
            if locate.clicked() {
                consume_recommendation_activation(ui);
                self.locate_specification_target(target);
            }
        });
        ui.collapsing(
            tr(
                self.locale,
                "规则与技术依据 / SI",
                "Rule and technical evidence / SI",
            ),
            |ui| {
                ui.label(format!(
                    "{:?} v{} · r{}",
                    recommendation.key.rule,
                    recommendation.key.rule_version,
                    recommendation.identity.input_revision
                ));
                ui.label(format!("{:?}", recommendation.action));
            },
        );
        ui.small(tr(self.locale, "不自动运行。局部问题清零不改变旧结果 / 输出资格。", "No automatic solve. Clearing local problems does not restore stale results or export eligibility."));
        true
    }
}

fn reveal_focused_action(response: &egui::Response) {
    if response.gained_focus() {
        response.scroll_to_me(Some(egui::Align::Center));
    }
}
