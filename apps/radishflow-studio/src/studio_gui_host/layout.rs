use super::*;

impl StudioGuiHost {
    pub fn update_window_layout(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        mutation: StudioGuiWindowLayoutMutation,
    ) -> RfResult<StudioGuiHostWindowLayoutUpdateResult> {
        self.validate_registered_window_for_layout(window_id, "layout updates")?;
        let snapshot = self.snapshot();
        let layout_state = self
            .layout_state_for_window_from_snapshot(&snapshot, window_id)
            .applying_mutation(&mutation);
        self.clear_window_drop_preview_for_scope(&layout_state.scope.layout_key);
        if let Some(legacy_layout_key) = layout_state.scope.legacy_layout_key() {
            self.layout_state_overrides.remove(&legacy_layout_key);
            self.window_drop_previews.remove(&legacy_layout_key);
        }
        self.layout_state_overrides.insert(
            layout_state.scope.layout_key.clone(),
            layout_state.persistence_state(),
        );
        self.persist_window_layouts()?;

        Ok(StudioGuiHostWindowLayoutUpdateResult {
            target_window_id: layout_state.scope.window_id,
            mutation,
            layout_state,
        })
    }

    pub fn query_window_drop_target(
        &self,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    ) -> RfResult<StudioGuiHostWindowDropTargetQueryResult> {
        self.validate_registered_window_for_layout(window_id, "drop target queries")?;
        let snapshot = self.snapshot();
        let layout_state = self.layout_state_for_window_from_snapshot(&snapshot, window_id);
        let drop_target = layout_state.drop_target_for_query(&query);
        let preview_layout_state = layout_state.preview_layout_state_for_query(&query);
        let preview_window = preview_layout_state.as_ref().map(|preview_layout_state| {
            snapshot
                .window_model_for_window(layout_state.scope.window_id)
                .with_layout_state(preview_layout_state.clone())
        });

        Ok(StudioGuiHostWindowDropTargetQueryResult {
            target_window_id: layout_state.scope.window_id,
            query,
            layout_state,
            drop_target,
            preview_layout_state,
            preview_window,
        })
    }

    pub fn set_window_drop_target_preview(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    ) -> RfResult<StudioGuiHostWindowDropTargetQueryResult> {
        let query_result = self.query_window_drop_target(window_id, query)?;
        self.clear_window_drop_preview_for_scope(&query_result.layout_state.scope.layout_key);
        if let (Some(drop_target), Some(preview_layout_state)) = (
            query_result.drop_target.clone(),
            query_result.preview_layout_state.clone(),
        ) {
            self.window_drop_previews.insert(
                query_result.layout_state.scope.layout_key.clone(),
                StudioGuiWindowDropPreviewState {
                    query,
                    drop_target,
                    preview_layout_state,
                },
            );
        }
        Ok(query_result)
    }

    pub fn clear_window_drop_target_preview(
        &mut self,
        window_id: Option<StudioWindowHostId>,
    ) -> RfResult<StudioGuiHostWindowDropPreviewClearResult> {
        self.validate_registered_window_for_layout(window_id, "drop preview updates")?;
        let snapshot = self.snapshot();
        let layout_state = self.layout_state_for_window_from_snapshot(&snapshot, window_id);
        let mut had_preview =
            self.clear_window_drop_preview_for_scope(&layout_state.scope.layout_key);
        if let Some(legacy_layout_key) = layout_state.scope.legacy_layout_key() {
            had_preview |= self.clear_window_drop_preview_for_scope(&legacy_layout_key);
        }
        Ok(StudioGuiHostWindowDropPreviewClearResult {
            target_window_id: layout_state.scope.window_id,
            layout_state,
            had_preview,
        })
    }

    pub fn apply_window_drop_target(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    ) -> RfResult<StudioGuiHostWindowDropTargetApplyResult> {
        let query_result = self.query_window_drop_target(window_id, query)?;
        let drop_target = query_result.drop_target.ok_or_else(|| {
            RfError::invalid_input(format!(
                "drop target query `{query:?}` is not applicable for the current layout state"
            ))
        })?;
        let mutation = query.layout_mutation();
        let update = self.update_window_layout(window_id, mutation.clone())?;

        Ok(StudioGuiHostWindowDropTargetApplyResult {
            target_window_id: update.target_window_id,
            query,
            mutation,
            drop_target,
            layout_state: update.layout_state,
        })
    }

    pub(super) fn persist_window_layouts(&self) -> RfResult<()> {
        match self.controller.document_path() {
            Some(project_path) => {
                save_persisted_window_layouts(project_path, &self.layout_state_overrides)
            }
            None => Ok(()),
        }
    }

    pub(super) fn layout_state_for_window_from_snapshot(
        &self,
        snapshot: &StudioGuiSnapshot,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowLayoutState {
        let derived = StudioGuiWindowLayoutState::from_snapshot_for_window(snapshot, window_id);
        self.layout_state_overrides
            .get(&derived.scope.layout_key)
            .or_else(|| {
                derived
                    .scope
                    .legacy_layout_key()
                    .as_ref()
                    .and_then(|legacy_layout_key| self.layout_state_overrides.get(legacy_layout_key))
            })
            .map(|persisted| derived.merged_with_persisted(persisted))
            .unwrap_or(derived)
    }

    pub(super) fn validate_registered_window_for_layout(
        &self,
        window_id: Option<StudioWindowHostId>,
        action: &str,
    ) -> RfResult<()> {
        if let Some(window_id) =
            window_id.filter(|window_id| self.state().window(*window_id).is_none())
        {
            return Err(RfError::invalid_input(format!(
                "window host `{window_id}` is not registered for {action}"
            )));
        }
        Ok(())
    }

    pub(super) fn clear_window_drop_preview_for_scope(&mut self, layout_key: &str) -> bool {
        self.window_drop_previews.remove(layout_key).is_some()
    }
}
