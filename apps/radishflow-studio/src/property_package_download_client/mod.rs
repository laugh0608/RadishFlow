use std::path::Path;
use std::time::Duration;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReqwestPropertyPackageDownloadHttpTransportOptions {
    pub request_timeout: Duration,
    pub user_agent: String,
}

impl Default for ReqwestPropertyPackageDownloadHttpTransportOptions {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            user_agent: format!("radishflow-studio/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReqwestPropertyPackageDownloadHttpTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestPropertyPackageDownloadHttpTransport {
    pub fn new() -> RfResult<Self> {
        Self::with_options(ReqwestPropertyPackageDownloadHttpTransportOptions::default())
    }

    pub fn with_options(
        options: ReqwestPropertyPackageDownloadHttpTransportOptions,
    ) -> RfResult<Self> {
        if options.request_timeout == Duration::ZERO {
            return Err(RfError::invalid_input(
                "reqwest property package download transport timeout must be greater than zero",
            ));
        }

        if options.user_agent.trim().is_empty() {
            return Err(RfError::invalid_input(
                "reqwest property package download transport user_agent must not be empty",
            ));
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(options.request_timeout)
            .user_agent(options.user_agent)
            .build()
            .map_err(|error| {
                RfError::invalid_input(format!(
                    "build reqwest property package download transport: {error}"
                ))
            })?;

        Ok(Self { client })
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

impl PropertyPackageDownloadHttpTransport for ReqwestPropertyPackageDownloadHttpTransport {
    fn send(
        &self,
        request: &PropertyPackageDownloadHttpRequest,
    ) -> Result<PropertyPackageDownloadHttpResponse, PropertyPackageDownloadHttpTransportError>
    {
        let accept_header = request.accept_content_types.join(", ");
        let response = self
            .client
            .get(&request.url)
            .header(reqwest::header::ACCEPT, accept_header)
            .send()
            .map_err(map_reqwest_transport_error)?;

        let status_code = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let received_at = SystemTime::now();
        let body = response.text().map_err(map_reqwest_transport_error)?;

        Ok(PropertyPackageDownloadHttpResponse {
            status_code,
            body,
            content_type,
            received_at,
        })
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

fn map_reqwest_transport_error(
    error: reqwest::Error,
) -> PropertyPackageDownloadHttpTransportError {
    if error.is_connect() {
        PropertyPackageDownloadHttpTransportError::connection_unavailable(error.to_string())
    } else if error.is_timeout() {
        PropertyPackageDownloadHttpTransportError::timeout(error.to_string())
    } else if error.is_body() {
        PropertyPackageDownloadHttpTransportError::other_transient(error.to_string())
    } else {
        PropertyPackageDownloadHttpTransportError::other_permanent(error.to_string())
    }
}

#[cfg(test)]
mod tests;
