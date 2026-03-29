use std::path::Path;
use std::time::SystemTime;

use rf_store::StoredAuthCacheIndex;
use rf_types::{RfError, RfResult};
use rf_ui::{PropertyPackageLeaseGrant, PropertyPackageManifest};

use crate::persist_downloaded_package_response_to_cache;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageDownloadResponse {
    pub contents: String,
    pub downloaded_at: SystemTime,
}

impl PropertyPackageDownloadResponse {
    pub fn new(contents: impl Into<String>, downloaded_at: SystemTime) -> Self {
        Self {
            contents: contents.into(),
            downloaded_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyPackageDownloadFetchErrorKind {
    Timeout,
    ConnectionUnavailable,
    RateLimited,
    ServiceUnavailable,
    Unauthorized,
    Forbidden,
    NotFound,
    InvalidResponse,
    OtherTransient,
    OtherPermanent,
}

impl PropertyPackageDownloadFetchErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::ConnectionUnavailable => "connection-unavailable",
            Self::RateLimited => "rate-limited",
            Self::ServiceUnavailable => "service-unavailable",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not-found",
            Self::InvalidResponse => "invalid-response",
            Self::OtherTransient => "other-transient",
            Self::OtherPermanent => "other-permanent",
        }
    }

    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::Timeout
                | Self::ConnectionUnavailable
                | Self::RateLimited
                | Self::ServiceUnavailable
                | Self::OtherTransient
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageDownloadFetchError {
    pub kind: PropertyPackageDownloadFetchErrorKind,
    pub message: String,
}

impl PropertyPackageDownloadFetchError {
    pub fn new(kind: PropertyPackageDownloadFetchErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(PropertyPackageDownloadFetchErrorKind::Timeout, message)
    }

    pub fn connection_unavailable(message: impl Into<String>) -> Self {
        Self::new(
            PropertyPackageDownloadFetchErrorKind::ConnectionUnavailable,
            message,
        )
    }

    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::new(PropertyPackageDownloadFetchErrorKind::RateLimited, message)
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(
            PropertyPackageDownloadFetchErrorKind::ServiceUnavailable,
            message,
        )
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(PropertyPackageDownloadFetchErrorKind::Unauthorized, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(PropertyPackageDownloadFetchErrorKind::Forbidden, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(PropertyPackageDownloadFetchErrorKind::NotFound, message)
    }

    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::new(
            PropertyPackageDownloadFetchErrorKind::InvalidResponse,
            message,
        )
    }

    pub fn other_transient(message: impl Into<String>) -> Self {
        Self::new(
            PropertyPackageDownloadFetchErrorKind::OtherTransient,
            message,
        )
    }

    pub fn other_permanent(message: impl Into<String>) -> Self {
        Self::new(
            PropertyPackageDownloadFetchErrorKind::OtherPermanent,
            message,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyPackageDownloadRetryPolicy {
    max_attempts: u32,
}

impl PropertyPackageDownloadRetryPolicy {
    pub const fn single_attempt() -> Self {
        Self { max_attempts: 1 }
    }

    pub fn new(max_attempts: u32) -> RfResult<Self> {
        if max_attempts == 0 {
            return Err(RfError::invalid_input(
                "property package download retry policy must allow at least one attempt",
            ));
        }

        Ok(Self { max_attempts })
    }

    pub const fn max_attempts(self) -> u32 {
        self.max_attempts
    }
}

impl Default for PropertyPackageDownloadRetryPolicy {
    fn default() -> Self {
        Self { max_attempts: 3 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageDownloadHttpRequest {
    pub url: String,
    pub accept_content_types: Vec<String>,
}

impl PropertyPackageDownloadHttpRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            accept_content_types: vec!["application/json".to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageDownloadHttpResponse {
    pub status_code: u16,
    pub body: String,
    pub content_type: Option<String>,
    pub received_at: SystemTime,
}

impl PropertyPackageDownloadHttpResponse {
    pub fn new(status_code: u16, body: impl Into<String>, received_at: SystemTime) -> Self {
        Self {
            status_code,
            body: body.into(),
            content_type: Some("application/json".to_string()),
            received_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyPackageDownloadHttpTransportErrorKind {
    Timeout,
    ConnectionUnavailable,
    OtherTransient,
    OtherPermanent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageDownloadHttpTransportError {
    pub kind: PropertyPackageDownloadHttpTransportErrorKind,
    pub message: String,
}

impl PropertyPackageDownloadHttpTransportError {
    pub fn new(
        kind: PropertyPackageDownloadHttpTransportErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(
            PropertyPackageDownloadHttpTransportErrorKind::Timeout,
            message,
        )
    }

    pub fn connection_unavailable(message: impl Into<String>) -> Self {
        Self::new(
            PropertyPackageDownloadHttpTransportErrorKind::ConnectionUnavailable,
            message,
        )
    }

    pub fn other_transient(message: impl Into<String>) -> Self {
        Self::new(
            PropertyPackageDownloadHttpTransportErrorKind::OtherTransient,
            message,
        )
    }

    pub fn other_permanent(message: impl Into<String>) -> Self {
        Self::new(
            PropertyPackageDownloadHttpTransportErrorKind::OtherPermanent,
            message,
        )
    }
}

pub trait PropertyPackageDownloadHttpTransport {
    fn send(
        &self,
        request: &PropertyPackageDownloadHttpRequest,
    ) -> Result<PropertyPackageDownloadHttpResponse, PropertyPackageDownloadHttpTransportError>;
}

impl<Transport> PropertyPackageDownloadHttpTransport for &Transport
where
    Transport: PropertyPackageDownloadHttpTransport,
{
    fn send(
        &self,
        request: &PropertyPackageDownloadHttpRequest,
    ) -> Result<PropertyPackageDownloadHttpResponse, PropertyPackageDownloadHttpTransportError>
    {
        (*self).send(request)
    }
}

#[derive(Debug, Clone)]
pub struct HttpPropertyPackageDownloadFetcher<Transport> {
    transport: Transport,
}

impl<Transport> HttpPropertyPackageDownloadFetcher<Transport> {
    pub fn new(transport: Transport) -> Self {
        Self { transport }
    }
}

pub trait PropertyPackageDownloadFetcher {
    fn fetch_download(
        &self,
        lease_grant: &PropertyPackageLeaseGrant,
    ) -> Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError>;
}

impl<Transport> PropertyPackageDownloadFetcher for HttpPropertyPackageDownloadFetcher<Transport>
where
    Transport: PropertyPackageDownloadHttpTransport,
{
    fn fetch_download(
        &self,
        lease_grant: &PropertyPackageLeaseGrant,
    ) -> Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError> {
        let request = property_package_download_http_request(lease_grant)?;
        let response = self
            .transport
            .send(&request)
            .map_err(map_http_transport_error)?;

        property_package_download_response_from_http(lease_grant, response)
    }
}

pub fn download_property_package_to_cache<Fetcher>(
    cache_root: impl AsRef<Path>,
    index: &mut StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    lease_grant: &PropertyPackageLeaseGrant,
    fetcher: &Fetcher,
) -> RfResult<()>
where
    Fetcher: PropertyPackageDownloadFetcher,
{
    download_property_package_to_cache_with_retry_policy(
        cache_root,
        index,
        manifest,
        lease_grant,
        fetcher,
        PropertyPackageDownloadRetryPolicy::default(),
    )
}

pub fn download_property_package_to_cache_with_retry_policy<Fetcher>(
    cache_root: impl AsRef<Path>,
    index: &mut StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    lease_grant: &PropertyPackageLeaseGrant,
    fetcher: &Fetcher,
    retry_policy: PropertyPackageDownloadRetryPolicy,
) -> RfResult<()>
where
    Fetcher: PropertyPackageDownloadFetcher,
{
    let response = fetch_download_with_retry_policy(fetcher, lease_grant, retry_policy)?;
    if response.contents.trim().is_empty() {
        return Err(RfError::invalid_input(format!(
            "download response for package `{}` must not be empty",
            lease_grant.package_id
        )));
    }

    persist_downloaded_package_response_to_cache(
        cache_root,
        index,
        manifest,
        lease_grant,
        &response.contents,
        response.downloaded_at,
    )
}

fn fetch_download_with_retry_policy<Fetcher>(
    fetcher: &Fetcher,
    lease_grant: &PropertyPackageLeaseGrant,
    retry_policy: PropertyPackageDownloadRetryPolicy,
) -> RfResult<PropertyPackageDownloadResponse>
where
    Fetcher: PropertyPackageDownloadFetcher,
{
    let max_attempts = retry_policy.max_attempts();

    for attempt in 1..=max_attempts {
        match fetcher.fetch_download(lease_grant) {
            Ok(response) => return Ok(response),
            Err(error) if error.kind.is_retryable() && attempt < max_attempts => {
                continue;
            }
            Err(error) => {
                let summary = if error.kind.is_retryable() {
                    format!(
                        "download for package `{}` exhausted {} attempts with {} error: {}",
                        lease_grant.package_id,
                        max_attempts,
                        error.kind.as_str(),
                        error.message
                    )
                } else {
                    format!(
                        "download for package `{}` failed on attempt {} with non-retryable {} error: {}",
                        lease_grant.package_id,
                        attempt,
                        error.kind.as_str(),
                        error.message
                    )
                };
                return Err(RfError::invalid_input(summary));
            }
        }
    }

    Err(RfError::invalid_input(format!(
        "download for package `{}` did not execute any fetch attempts",
        lease_grant.package_id
    )))
}

fn property_package_download_http_request(
    lease_grant: &PropertyPackageLeaseGrant,
) -> Result<PropertyPackageDownloadHttpRequest, PropertyPackageDownloadFetchError> {
    if lease_grant.download_url.trim().is_empty() {
        return Err(PropertyPackageDownloadFetchError::invalid_response(
            "lease grant download_url must not be empty",
        ));
    }

    Ok(PropertyPackageDownloadHttpRequest::new(
        lease_grant.download_url.clone(),
    ))
}

fn map_http_transport_error(
    error: PropertyPackageDownloadHttpTransportError,
) -> PropertyPackageDownloadFetchError {
    match error.kind {
        PropertyPackageDownloadHttpTransportErrorKind::Timeout => {
            PropertyPackageDownloadFetchError::timeout(error.message)
        }
        PropertyPackageDownloadHttpTransportErrorKind::ConnectionUnavailable => {
            PropertyPackageDownloadFetchError::connection_unavailable(error.message)
        }
        PropertyPackageDownloadHttpTransportErrorKind::OtherTransient => {
            PropertyPackageDownloadFetchError::other_transient(error.message)
        }
        PropertyPackageDownloadHttpTransportErrorKind::OtherPermanent => {
            PropertyPackageDownloadFetchError::other_permanent(error.message)
        }
    }
}

fn property_package_download_response_from_http(
    lease_grant: &PropertyPackageLeaseGrant,
    response: PropertyPackageDownloadHttpResponse,
) -> Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError> {
    match response.status_code {
        200..=299 => {
            if let Some(content_type) = &response.content_type
                && !content_type
                    .to_ascii_lowercase()
                    .contains("application/json")
            {
                return Err(PropertyPackageDownloadFetchError::invalid_response(
                    format!(
                        "download for package `{}` returned unsupported content-type `{content_type}`",
                        lease_grant.package_id
                    ),
                ));
            }

            if response.body.trim().is_empty() {
                return Err(PropertyPackageDownloadFetchError::invalid_response(
                    format!(
                        "download for package `{}` returned an empty successful body",
                        lease_grant.package_id
                    ),
                ));
            }

            Ok(PropertyPackageDownloadResponse::new(
                response.body,
                response.received_at,
            ))
        }
        401 => Err(PropertyPackageDownloadFetchError::unauthorized(format!(
            "download lease for package `{}` is no longer authorized",
            lease_grant.package_id
        ))),
        403 => Err(PropertyPackageDownloadFetchError::forbidden(format!(
            "download lease for package `{}` was rejected by asset delivery",
            lease_grant.package_id
        ))),
        404 => Err(PropertyPackageDownloadFetchError::not_found(format!(
            "download resource for package `{}` was not found",
            lease_grant.package_id
        ))),
        408 | 504 => Err(PropertyPackageDownloadFetchError::timeout(format!(
            "download for package `{}` timed out with HTTP {}",
            lease_grant.package_id, response.status_code
        ))),
        429 => Err(PropertyPackageDownloadFetchError::rate_limited(format!(
            "download for package `{}` was rate limited",
            lease_grant.package_id
        ))),
        500..=599 => Err(PropertyPackageDownloadFetchError::service_unavailable(
            format!(
                "download for package `{}` failed with server status {}",
                lease_grant.package_id, response.status_code
            ),
        )),
        _ => Err(PropertyPackageDownloadFetchError::invalid_response(
            format!(
                "download for package `{}` returned unsupported HTTP status {}",
                lease_grant.package_id, response.status_code
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
        property_package_payload_integrity, read_property_package_payload,
    };
    use rf_types::ComponentId;
    use rf_ui::{PropertyPackageLeaseGrant, PropertyPackageManifest, PropertyPackageSource};

    use crate::{
        HttpPropertyPackageDownloadFetcher, PropertyPackageDownloadFetchError,
        PropertyPackageDownloadFetchErrorKind, PropertyPackageDownloadFetcher,
        PropertyPackageDownloadHttpRequest, PropertyPackageDownloadHttpResponse,
        PropertyPackageDownloadHttpTransport, PropertyPackageDownloadHttpTransportError,
        PropertyPackageDownloadResponse, PropertyPackageDownloadRetryPolicy,
        download_property_package_to_cache, download_property_package_to_cache_with_retry_policy,
        parse_property_package_download_json,
    };

    struct StaticDownloadFetcher {
        response: PropertyPackageDownloadResponse,
    }

    impl PropertyPackageDownloadFetcher for StaticDownloadFetcher {
        fn fetch_download(
            &self,
            _lease_grant: &PropertyPackageLeaseGrant,
        ) -> Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError> {
            Ok(self.response.clone())
        }
    }

    struct ScriptedDownloadFetcher {
        responses: RefCell<
            Vec<Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError>>,
        >,
        call_count: Cell<u32>,
    }

    impl ScriptedDownloadFetcher {
        fn new(
            responses: Vec<
                Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError>,
            >,
        ) -> Self {
            Self {
                responses: RefCell::new(responses),
                call_count: Cell::new(0),
            }
        }

        fn call_count(&self) -> u32 {
            self.call_count.get()
        }
    }

    impl PropertyPackageDownloadFetcher for ScriptedDownloadFetcher {
        fn fetch_download(
            &self,
            _lease_grant: &PropertyPackageLeaseGrant,
        ) -> Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError> {
            self.call_count.set(self.call_count.get() + 1);
            self.responses.borrow_mut().remove(0)
        }
    }

    struct ScriptedHttpTransport {
        responses: RefCell<
            Vec<
                Result<
                    PropertyPackageDownloadHttpResponse,
                    PropertyPackageDownloadHttpTransportError,
                >,
            >,
        >,
        call_count: Cell<u32>,
        requests: RefCell<Vec<PropertyPackageDownloadHttpRequest>>,
    }

    impl ScriptedHttpTransport {
        fn new(
            responses: Vec<
                Result<
                    PropertyPackageDownloadHttpResponse,
                    PropertyPackageDownloadHttpTransportError,
                >,
            >,
        ) -> Self {
            Self {
                responses: RefCell::new(responses),
                call_count: Cell::new(0),
                requests: RefCell::new(Vec::new()),
            }
        }

        fn call_count(&self) -> u32 {
            self.call_count.get()
        }

        fn requests(&self) -> Vec<PropertyPackageDownloadHttpRequest> {
            self.requests.borrow().clone()
        }
    }

    impl PropertyPackageDownloadHttpTransport for ScriptedHttpTransport {
        fn send(
            &self,
            request: &PropertyPackageDownloadHttpRequest,
        ) -> Result<PropertyPackageDownloadHttpResponse, PropertyPackageDownloadHttpTransportError>
        {
            self.call_count.set(self.call_count.get() + 1);
            self.requests.borrow_mut().push(request.clone());
            self.responses.borrow_mut().remove(0)
        }
    }

    #[test]
    fn download_property_package_to_cache_fetches_response_and_persists_assets() {
        let root = unique_temp_path("download-fetcher-success");
        let mut index = sample_auth_cache_index();
        let download = parse_property_package_download_json(&sample_download_json())
            .expect("expected sample download");
        let payload = download
            .to_stored_payload()
            .expect("expected sample payload");
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
        let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
        let fetcher = StaticDownloadFetcher {
            response: PropertyPackageDownloadResponse::new(sample_download_json(), timestamp(200)),
        };

        download_property_package_to_cache(&root, &mut index, &manifest, &lease_grant, &fetcher)
            .expect("expected cached download");

        let cached_record = &index.property_packages[0];
        let payload = read_property_package_payload(
            cached_record
                .payload_path_under(&root)
                .expect("expected payload path"),
        )
        .expect("expected payload read");

        assert_eq!(payload.package_id, "binary-hydrocarbon-lite-v1");
        assert_eq!(payload.components.len(), 2);

        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn download_property_package_to_cache_rejects_hash_mismatch_before_updating_index() {
        let root = unique_temp_path("download-fetcher-mismatch");
        let mut index = sample_auth_cache_index();
        let download = parse_property_package_download_json(&sample_download_json())
            .expect("expected sample download");
        let payload = download
            .to_stored_payload()
            .expect("expected sample payload");
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
        let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
        let fetcher = StaticDownloadFetcher {
            response: PropertyPackageDownloadResponse::new(
                sample_download_json().replace("\"Methane\"", "\"Methane Modified\""),
                timestamp(200),
            ),
        };

        let error = download_property_package_to_cache(
            &root,
            &mut index,
            &manifest,
            &lease_grant,
            &fetcher,
        )
        .expect_err("expected hash mismatch");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(error.message().contains("payload hash"));
        assert!(index.property_packages.is_empty());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn download_property_package_to_cache_retries_retryable_errors_before_success() {
        let root = unique_temp_path("download-fetcher-retryable");
        let mut index = sample_auth_cache_index();
        let download = parse_property_package_download_json(&sample_download_json())
            .expect("expected sample download");
        let payload = download
            .to_stored_payload()
            .expect("expected sample payload");
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
        let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
        let fetcher = ScriptedDownloadFetcher::new(vec![
            Err(PropertyPackageDownloadFetchError::timeout(
                "adapter timed out",
            )),
            Err(PropertyPackageDownloadFetchError::service_unavailable(
                "asset delivery is warming up",
            )),
            Ok(PropertyPackageDownloadResponse::new(
                sample_download_json(),
                timestamp(200),
            )),
        ]);

        download_property_package_to_cache(&root, &mut index, &manifest, &lease_grant, &fetcher)
            .expect("expected cached download after retries");

        assert_eq!(fetcher.call_count(), 3);
        assert_eq!(index.property_packages.len(), 1);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn download_property_package_to_cache_does_not_retry_non_retryable_errors() {
        let root = unique_temp_path("download-fetcher-non-retryable");
        let mut index = sample_auth_cache_index();
        let download = parse_property_package_download_json(&sample_download_json())
            .expect("expected sample download");
        let payload = download
            .to_stored_payload()
            .expect("expected sample payload");
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
        let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
        let fetcher = ScriptedDownloadFetcher::new(vec![
            Err(PropertyPackageDownloadFetchError::unauthorized(
                "lease is no longer valid",
            )),
            Ok(PropertyPackageDownloadResponse::new(
                sample_download_json(),
                timestamp(200),
            )),
        ]);

        let error = download_property_package_to_cache(
            &root,
            &mut index,
            &manifest,
            &lease_grant,
            &fetcher,
        )
        .expect_err("expected non-retryable fetch failure");

        assert_eq!(fetcher.call_count(), 1);
        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(error.message().contains("non-retryable unauthorized error"));
        assert!(index.property_packages.is_empty());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn download_property_package_to_cache_reports_retry_exhaustion() {
        let root = unique_temp_path("download-fetcher-exhausted");
        let mut index = sample_auth_cache_index();
        let download = parse_property_package_download_json(&sample_download_json())
            .expect("expected sample download");
        let payload = download
            .to_stored_payload()
            .expect("expected sample payload");
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
        let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
        let fetcher = ScriptedDownloadFetcher::new(vec![
            Err(PropertyPackageDownloadFetchError::rate_limited(
                "retry later",
            )),
            Err(PropertyPackageDownloadFetchError::rate_limited(
                "retry later again",
            )),
        ]);
        let retry_policy =
            PropertyPackageDownloadRetryPolicy::new(2).expect("expected retry policy");

        let error = download_property_package_to_cache_with_retry_policy(
            &root,
            &mut index,
            &manifest,
            &lease_grant,
            &fetcher,
            retry_policy,
        )
        .expect_err("expected retry exhaustion");

        assert_eq!(fetcher.call_count(), 2);
        assert!(error.message().contains("exhausted 2 attempts"));
        assert!(index.property_packages.is_empty());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn http_fetcher_builds_request_from_lease_grant() {
        let transport = ScriptedHttpTransport::new(vec![Ok(
            PropertyPackageDownloadHttpResponse::new(200, sample_download_json(), timestamp(200)),
        )]);
        let fetcher = HttpPropertyPackageDownloadFetcher::new(&transport);

        let response = fetcher
            .fetch_download(&sample_lease_grant("sha256:test", 1))
            .expect("expected http fetch success");

        assert_eq!(response.contents, sample_download_json());
        assert_eq!(transport.call_count(), 1);
        assert_eq!(transport.requests().len(), 1);
        assert_eq!(
            transport.requests()[0].url,
            "https://assets.radish.local/lease-1"
        );
        assert_eq!(
            transport.requests()[0].accept_content_types,
            vec!["application/json".to_string()]
        );
    }

    #[test]
    fn http_fetcher_maps_http_statuses_into_existing_failure_categories() {
        let cases = [
            (401, PropertyPackageDownloadFetchErrorKind::Unauthorized),
            (403, PropertyPackageDownloadFetchErrorKind::Forbidden),
            (404, PropertyPackageDownloadFetchErrorKind::NotFound),
            (408, PropertyPackageDownloadFetchErrorKind::Timeout),
            (429, PropertyPackageDownloadFetchErrorKind::RateLimited),
            (
                503,
                PropertyPackageDownloadFetchErrorKind::ServiceUnavailable,
            ),
            (504, PropertyPackageDownloadFetchErrorKind::Timeout),
            (302, PropertyPackageDownloadFetchErrorKind::InvalidResponse),
        ];

        for (status_code, expected_kind) in cases {
            let transport = ScriptedHttpTransport::new(vec![Ok(
                PropertyPackageDownloadHttpResponse::new(status_code, "{}", timestamp(200)),
            )]);
            let fetcher = HttpPropertyPackageDownloadFetcher::new(&transport);

            let error = fetcher
                .fetch_download(&sample_lease_grant("sha256:test", 1))
                .expect_err("expected mapped http failure");

            assert_eq!(error.kind, expected_kind);
        }
    }

    #[test]
    fn http_fetcher_maps_transport_errors_into_existing_failure_categories() {
        let cases = [
            (
                PropertyPackageDownloadHttpTransportError::timeout("timed out"),
                PropertyPackageDownloadFetchErrorKind::Timeout,
            ),
            (
                PropertyPackageDownloadHttpTransportError::connection_unavailable("offline"),
                PropertyPackageDownloadFetchErrorKind::ConnectionUnavailable,
            ),
            (
                PropertyPackageDownloadHttpTransportError::other_transient("proxy reset"),
                PropertyPackageDownloadFetchErrorKind::OtherTransient,
            ),
            (
                PropertyPackageDownloadHttpTransportError::other_permanent("bad TLS config"),
                PropertyPackageDownloadFetchErrorKind::OtherPermanent,
            ),
        ];

        for (transport_error, expected_kind) in cases {
            let transport = ScriptedHttpTransport::new(vec![Err(transport_error)]);
            let fetcher = HttpPropertyPackageDownloadFetcher::new(&transport);

            let error = fetcher
                .fetch_download(&sample_lease_grant("sha256:test", 1))
                .expect_err("expected mapped transport failure");

            assert_eq!(error.kind, expected_kind);
        }
    }

    #[test]
    fn http_fetcher_rejects_success_without_json_content_type() {
        let mut response =
            PropertyPackageDownloadHttpResponse::new(200, sample_download_json(), timestamp(200));
        response.content_type = Some("application/octet-stream".to_string());
        let transport = ScriptedHttpTransport::new(vec![Ok(response)]);
        let fetcher = HttpPropertyPackageDownloadFetcher::new(&transport);

        let error = fetcher
            .fetch_download(&sample_lease_grant("sha256:test", 1))
            .expect_err("expected invalid response content type");

        assert_eq!(
            error.kind,
            PropertyPackageDownloadFetchErrorKind::InvalidResponse
        );
    }

    fn sample_auth_cache_index() -> StoredAuthCacheIndex {
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-credential"),
        );
        index.entitlement = Some(StoredEntitlementCache {
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            synced_at: timestamp(100),
            issued_at: timestamp(90),
            expires_at: timestamp(500),
            offline_lease_expires_at: Some(timestamp(900)),
            feature_keys: BTreeSet::from(["desktop-login".to_string()]),
            allowed_package_ids: BTreeSet::from(["binary-hydrocarbon-lite-v1".to_string()]),
        });
        index
    }

    fn sample_manifest(hash: &str, size_bytes: u64) -> PropertyPackageManifest {
        let mut manifest = PropertyPackageManifest::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            PropertyPackageSource::RemoteDerivedPackage,
        );
        manifest.hash = hash.to_string();
        manifest.size_bytes = size_bytes;
        manifest.component_ids = vec![ComponentId::new("methane"), ComponentId::new("ethane")];
        manifest
    }

    fn sample_lease_grant(hash: &str, size_bytes: u64) -> PropertyPackageLeaseGrant {
        PropertyPackageLeaseGrant {
            package_id: "binary-hydrocarbon-lite-v1".to_string(),
            version: "2026.03.1".to_string(),
            lease_id: "lease-1".to_string(),
            download_url: "https://assets.radish.local/lease-1".to_string(),
            hash: hash.to_string(),
            size_bytes,
            expires_at: timestamp(210),
        }
    }

    fn sample_download_json() -> String {
        fs::read_to_string(sample_download_path()).expect("expected sample download json")
    }

    fn sample_download_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1/download.json")
    }

    fn timestamp(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
    }
}
