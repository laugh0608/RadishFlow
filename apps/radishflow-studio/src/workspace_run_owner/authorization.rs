use std::time::SystemTime;

use rf_ui::{AppState, AuthenticatedUser, EntitlementSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunAuthorizationRefusal {
    Expired,
    Changed,
}

/// Contains authorization facts, never access tokens or credential handles.
/// No-snapshot local sessions retain the existing local-run compatibility path.
#[derive(Debug)]
pub(super) struct RunAuthorization {
    authority: Option<String>,
    subject: Option<AuthenticatedUser>,
    authenticated_at: Option<SystemTime>,
    entitlement: Option<EntitlementSnapshot>,
    package_id: String,
}

impl RunAuthorization {
    pub(super) fn capture(
        app: &AppState,
        package_id: &str,
        now: SystemTime,
    ) -> Result<Self, RunAuthorizationRefusal> {
        let authorization = Self {
            authority: app.auth_session.authority_url.clone(),
            subject: app.auth_session.current_user.clone(),
            authenticated_at: app.auth_session.last_authenticated_at,
            entitlement: app.entitlement.snapshot.clone(),
            package_id: package_id.into(),
        };
        authorization.allow_start(app, now)?;
        Ok(authorization)
    }

    pub(super) fn allow_start(
        &self,
        app: &AppState,
        now: SystemTime,
    ) -> Result<(), RunAuthorizationRefusal> {
        self.check_revocation(app)?;
        for snapshot in [self.entitlement.as_ref(), app.entitlement.snapshot.as_ref()]
            .into_iter()
            .flatten()
        {
            if now
                >= snapshot
                    .offline_lease_expires_at
                    .unwrap_or(snapshot.expires_at)
            {
                return Err(RunAuthorizationRefusal::Expired);
            }
        }
        Ok(())
    }

    // Once start is granted, elapsed wall time alone never revokes its results.
    pub(super) fn check_revocation(&self, app: &AppState) -> Result<(), RunAuthorizationRefusal> {
        let same_subject = match (&self.subject, &app.auth_session.current_user) {
            (None, None) => true,
            (Some(old), Some(new)) => {
                old.subject_id == new.subject_id && old.tenant_id == new.tenant_id
            }
            _ => false,
        };
        let same_grant = match (&self.entitlement, &app.entitlement.snapshot) {
            (None, None) => true,
            (Some(old), Some(new)) => {
                old.subject_id == new.subject_id
                    && old.tenant_id == new.tenant_id
                    && new.allowed_package_ids.contains(&self.package_id)
            }
            _ => false,
        };
        if !same_subject
            || !same_grant
            || self.authority != app.auth_session.authority_url
            || self.authenticated_at != app.auth_session.last_authenticated_at
        {
            return Err(RunAuthorizationRefusal::Changed);
        }
        Ok(())
    }
}
